use std::collections::HashMap;

use diesel::{ExpressionMethods, QueryDsl, SelectableHelper, dsl::count_star};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::domain::public_author::PublicAuthor,
    persistence::public_authors::load_public_authors,
    schema::{post_tags, post_votes, posts, tags},
};

use super::super::{
    domain::{cache::CachedPostInfo, post::PostInfo, vote::VoteState},
    error::BlogError,
};
use super::{
    authority::has_current_blog_authority, blog_repository::BlogRepository, records::PostInfoRecord,
};

pub struct PostPage {
    pub posts: Vec<CachedPostInfo>,
    pub available_pages: usize,
}

pub struct PostPresentationData {
    pub authors: HashMap<Uuid, PublicAuthor>,
    pub votes: HashMap<Uuid, VoteState>,
}

impl BlogRepository {
    /// Whether the viewer currently holds the blog management permission.
    pub async fn can_manage_blog(&self, viewer_id: Option<Uuid>) -> Result<bool, BlogError> {
        if viewer_id.is_none() {
            return Ok(false);
        }
        let mut connection = self.connection().await?;
        has_current_blog_authority(&mut connection, viewer_id).await
    }

    pub async fn list_posts(
        &self,
        page: usize,
        page_size: usize,
        viewer_id: Option<Uuid>,
    ) -> Result<PostPage, BlogError> {
        let page = page.clamp(1, 10_000);
        let page_size = page_size.clamp(1, 100);
        let offset = page.saturating_sub(1).saturating_mul(page_size);
        let offset = i64::try_from(offset).unwrap_or(i64::MAX);
        let mut connection = self.connection().await?;
        let include_unpublished = has_current_blog_authority(&mut connection, viewer_id).await?;
        let mut count_query = posts::table.select(count_star()).into_boxed();
        if !include_unpublished {
            count_query = count_query.filter(posts::post_is_published.eq(true));
        }
        let total_rows = count_query.first::<i64>(&mut connection).await?;
        let mut query = posts::table
            .select(PostInfoRecord::as_select())
            .into_boxed();
        if !include_unpublished {
            query = query.filter(posts::post_is_published.eq(true));
        }
        let post_info = query
            .order((posts::post_created_at.desc(), posts::post_id.desc()))
            .offset(offset)
            .limit(i64::try_from(page_size).unwrap_or(100))
            .load::<PostInfoRecord>(&mut connection)
            .await?
            .into_iter()
            .map(PostInfo::from)
            .collect::<Vec<_>>();
        let post_ids = post_info
            .iter()
            .map(|post| post.post_id)
            .collect::<Vec<_>>();
        let tag_rows = load_tag_rows(&mut connection, &post_ids).await?;
        let total_rows = usize::try_from(total_rows).unwrap_or_default();
        Ok(PostPage {
            posts: combine_posts(post_info, tag_rows),
            available_pages: total_rows.div_ceil(page_size),
        })
    }

    pub async fn posts_by_ids(&self, post_ids: &[Uuid]) -> Result<Vec<CachedPostInfo>, BlogError> {
        if post_ids.is_empty() {
            return Ok(Vec::new());
        }
        let mut connection = self.connection().await?;
        let post_info = posts::table
            .filter(posts::post_id.eq_any(post_ids))
            .select(PostInfoRecord::as_select())
            .load::<PostInfoRecord>(&mut connection)
            .await?
            .into_iter()
            .map(PostInfo::from)
            .collect();
        let tag_rows = load_tag_rows(&mut connection, post_ids).await?;
        Ok(combine_posts(post_info, tag_rows))
    }

    pub async fn recent_posts(&self, limit: i64) -> Result<Vec<CachedPostInfo>, BlogError> {
        let mut connection = self.connection().await?;
        let post_info = posts::table
            .select(PostInfoRecord::as_select())
            .order((posts::post_created_at.desc(), posts::post_id.desc()))
            .limit(limit.clamp(1, 10_000))
            .load::<PostInfoRecord>(&mut connection)
            .await?
            .into_iter()
            .map(PostInfo::from)
            .collect::<Vec<_>>();
        let ids = post_info
            .iter()
            .map(|post| post.post_id)
            .collect::<Vec<_>>();
        let tag_rows = load_tag_rows(&mut connection, &ids).await?;
        Ok(combine_posts(post_info, tag_rows))
    }

    pub async fn presentation_data(
        &self,
        posts: &[CachedPostInfo],
        viewer_id: Option<Uuid>,
    ) -> Result<PostPresentationData, BlogError> {
        let mut user_ids = posts.iter().map(|post| post.user_id).collect::<Vec<_>>();
        user_ids.sort_unstable();
        user_ids.dedup();
        let post_ids = posts.iter().map(|post| post.post_id).collect::<Vec<_>>();
        let mut connection = self.connection().await?;
        let authors = load_public_authors(&mut connection, &user_ids).await?;
        let votes = match viewer_id {
            Some(viewer_id) if !post_ids.is_empty() => post_votes::table
                .filter(post_votes::post_id.eq_any(post_ids))
                .filter(post_votes::user_id.eq(viewer_id))
                .select((post_votes::post_id, post_votes::is_upvote))
                .load::<(Uuid, bool)>(&mut connection)
                .await?
                .into_iter()
                .map(|(post_id, upvote)| (post_id, vote_state(upvote)))
                .collect(),
            Some(_) | None => HashMap::new(),
        };
        Ok(PostPresentationData { authors, votes })
    }
}

async fn load_tag_rows(
    connection: &mut diesel_async::AsyncPgConnection,
    post_ids: &[Uuid],
) -> Result<Vec<(Uuid, String)>, diesel::result::Error> {
    if post_ids.is_empty() {
        return Ok(Vec::new());
    }
    post_tags::table
        .inner_join(tags::table)
        .filter(post_tags::post_id.eq_any(post_ids))
        .select((post_tags::post_id, tags::tag_name))
        .load(connection)
        .await
}

fn combine_posts(posts: Vec<PostInfo>, tag_rows: Vec<(Uuid, String)>) -> Vec<CachedPostInfo> {
    let mut tags_by_post = HashMap::<Uuid, Vec<String>>::new();
    for (post_id, tag) in tag_rows {
        tags_by_post.entry(post_id).or_default().push(tag);
    }
    posts
        .into_iter()
        .map(|post| {
            let tags = tags_by_post.remove(&post.post_id).unwrap_or_default();
            CachedPostInfo::from_post_info_with_tags(post, tags)
        })
        .collect()
}

pub(super) fn vote_state(upvote: bool) -> VoteState {
    if upvote {
        VoteState::Upvoted
    } else {
        VoteState::Downvoted
    }
}
