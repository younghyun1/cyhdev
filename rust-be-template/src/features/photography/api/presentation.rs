use crate::features::photography::{domain::social::{CommentPresentation, PhotographCommentResponse}, service::photography_service::PhotographyService};

pub(super) async fn comment_response(service: &PhotographyService, presentation: CommentPresentation) -> PhotographCommentResponse { service.present_comment(presentation).await }
