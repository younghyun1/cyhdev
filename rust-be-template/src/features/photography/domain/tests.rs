use chrono::Utc;
use uuid::Uuid;

use super::{
    photograph::{Photograph, PhotographContext},
    social::{MAX_PHOTOGRAPH_COMMENT_CHARS, PhotographComment, PhotographCommentBody, PhotographCommentBodyError, PhotographCommentResponse},
};
use crate::features::blog::domain::{post::UserBadgeInfo, vote::VoteState};

#[test]
fn deleted_owner_masks_photograph_identity_and_location_only() {
    let photograph_id = Uuid::now_v7();
    let mut photograph = Photograph {
        photograph_id,
        user_id: Uuid::now_v7(),
        photograph_shot_at: Some(Utc::now()),
        photograph_created_at: Utc::now(),
        photograph_updated_at: Utc::now(),
        photograph_image_type: 1,
        photograph_is_on_cloud: true,
        photograph_link: "https://example.invalid/photo".to_owned(),
        photograph_comments: "Retained caption".to_owned(),
        photograph_lat: 39.7392,
        photograph_lon: -104.9903,
        photograph_thumbnail_link: "https://example.invalid/thumb".to_owned(),
        photograph_context: PhotographContext::Photography,
        photograph_view_count: 21,
        photograph_total_upvotes: 9,
        photograph_total_downvotes: 3,
    };

    photograph.anonymize_deleted_owner();

    assert_eq!(photograph.photograph_id, photograph_id);
    assert_eq!(photograph.user_id, Uuid::nil());
    assert_eq!(photograph.photograph_lat, 0.0);
    assert_eq!(photograph.photograph_lon, 0.0);
    assert_eq!(photograph.photograph_comments, "Retained caption");
    assert_eq!(photograph.photograph_view_count, 21);
    assert_eq!(photograph.photograph_total_upvotes, 9);
    assert_eq!(photograph.photograph_total_downvotes, 3);
}

#[test]
fn deleted_comment_author_does_not_change_comment_content() {
    let comment_id = Uuid::now_v7();
    let photograph_id = Uuid::now_v7();
    let comment = PhotographComment {
        photograph_comment_id: comment_id,
        photograph_id,
        user_id: Uuid::now_v7(),
        photograph_comment_content: "Retained photograph comment".to_owned(),
        photograph_comment_created_at: Utc::now(),
        photograph_comment_updated_at: None,
        parent_photograph_comment_id: None,
        photograph_comment_total_upvotes: 5,
        photograph_comment_total_downvotes: 1,
    };

    let response = PhotographCommentResponse::from_comment_votestate_and_badge_info(
        comment,
        VoteState::DidNotVote,
        Uuid::nil(),
        UserBadgeInfo::deleted(),
    );

    assert_eq!(response.photograph_comment_id, comment_id);
    assert_eq!(response.photograph_id, photograph_id);
    assert_eq!(response.user_id, Uuid::nil());
    assert_eq!(
        response.photograph_comment_content,
        "Retained photograph comment"
    );
    assert_eq!(response.photograph_comment_total_upvotes, 5);
    assert_eq!(response.photograph_comment_total_downvotes, 1);
}

#[test]
fn comment_body_limit_counts_unicode_characters_instead_of_bytes() {
    let body = "🦀".repeat(MAX_PHOTOGRAPH_COMMENT_CHARS);
    assert!(PhotographCommentBody::parse(body).is_ok());
    let overflow = "界".repeat(MAX_PHOTOGRAPH_COMMENT_CHARS + 1);
    assert!(matches!(
        PhotographCommentBody::parse(overflow),
        Err(PhotographCommentBodyError::TooLong)
    ));
}

#[test]
fn comment_body_rejects_unicode_whitespace_only() {
    assert!(matches!(
        PhotographCommentBody::parse(" \n\t\u{2003}".to_owned()),
        Err(PhotographCommentBodyError::Empty)
    ));
}
