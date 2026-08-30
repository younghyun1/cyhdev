use chrono::Utc;
use uuid::Uuid;

use super::{
    cache::CachedPostInfo,
    comment::{BlogCommentBody, BlogCommentBodyError, Comment, CommentResponse, MAX_BLOG_COMMENT_CHARS},
    post::{PostInfoWithVote, UserBadgeInfo},
    vote::VoteState,
};
use crate::features::accounts::domain::account::DELETED_USER_DISPLAY_NAME;

#[test]
fn deleted_badge_contains_no_profile_or_country_identity() {
    let badge = UserBadgeInfo::deleted();
    assert_eq!(badge.user_name, DELETED_USER_DISPLAY_NAME);
    assert!(badge.user_profile_picture_url.is_empty());
    assert!(badge.user_country_flag.is_none());
}

#[test]
fn comment_limit_counts_unicode_characters() {
    assert!(BlogCommentBody::parse("🦀".repeat(MAX_BLOG_COMMENT_CHARS)).is_ok());
    assert!(matches!(
        BlogCommentBody::parse("界".repeat(MAX_BLOG_COMMENT_CHARS + 1)),
        Err(BlogCommentBodyError::TooLong)
    ));
}

#[test]
fn comment_rejects_unicode_whitespace_only() {
    assert!(matches!(
        BlogCommentBody::parse(" \n\t\u{2003}".to_owned()),
        Err(BlogCommentBodyError::Empty)
    ));
}

#[test]
fn retained_post_masks_only_author_identity() {
    let post_id = Uuid::now_v7();
    let post = CachedPostInfo {
        post_id,
        user_id: Uuid::now_v7(),
        post_title: "Retained title".to_owned(),
        post_slug: "retained-title".to_owned(),
        post_summary: Some("Retained summary".to_owned()),
        post_created_at: Utc::now(),
        post_updated_at: Utc::now(),
        post_published_at: Some(Utc::now()),
        post_is_published: true,
        post_view_count: 41,
        post_share_count: 7,
        total_upvotes: 13,
        total_downvotes: 2,
        post_tags: vec!["rust".to_owned()],
    };

    let response = PostInfoWithVote::from_cached_info_with_vote(
        post,
        VoteState::Upvoted,
        Uuid::nil(),
        UserBadgeInfo::deleted(),
    );
    assert_eq!(response.post_id, post_id);
    assert_eq!(response.user_id, Uuid::nil());
    assert_eq!(response.user_name, DELETED_USER_DISPLAY_NAME);
    assert!(response.user_profile_picture_url.is_empty());
    assert!(response.user_country_flag.is_none());
    assert_eq!(response.post_view_count, 41);
    assert_eq!(response.total_upvotes, 13);
    assert_eq!(response.total_downvotes, 2);
    assert_eq!(response.post_tags, vec!["rust"]);
    assert!(matches!(response.vote_state, VoteState::Upvoted));
}

#[test]
fn retained_comment_masks_author_without_changing_content_or_votes() {
    let comment_id = Uuid::now_v7();
    let post_id = Uuid::now_v7();
    let comment = Comment {
        comment_id,
        post_id,
        user_id: Uuid::now_v7(),
        comment_content: "Retained comment".to_owned(),
        comment_created_at: Utc::now(),
        comment_updated_at: None,
        parent_comment_id: None,
        total_upvotes: 8,
        total_downvotes: 1,
    };
    let response = CommentResponse::from_comment_votestate_and_badge_info(
        comment,
        VoteState::DidNotVote,
        Uuid::nil(),
        UserBadgeInfo::deleted(),
    );
    assert_eq!(response.comment_id, comment_id);
    assert_eq!(response.post_id, post_id);
    assert_eq!(response.user_id, Uuid::nil());
    assert_eq!(response.comment_content, "Retained comment");
    assert_eq!(response.total_upvotes, 8);
    assert_eq!(response.total_downvotes, 1);
}
