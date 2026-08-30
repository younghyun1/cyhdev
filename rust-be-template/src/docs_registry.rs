//! OpenAPI registry imports separated from the derive declaration.

pub(crate) use crate::dto::{
    requests::{
        admin::{
            authorization_request::{AssignRoleRequest, SetRolePermissionRequest},
            media_cleanup_request::ResolveMediaCleanupRequest,
            retention_notification_request::RetentionNotificationStatusRequest,
        },
        auth::{
            delete_account_request::DeleteAccountRequest,
            login_request::LoginRequest,
            oidc_request::{OidcLinkCompleteRequest, OidcUnlinkRequest},
            reset_password::ResetPasswordProcessRequest,
            reset_password_request::ResetPasswordRequest,
            signup_request::SignupRequest,
            update_profile_request::UpdateProfileRequest,
            verify_user_email_request::VerifyUserEmailRequest,
        },
        blog::{
            get_posts_request::GetPostsRequest, submit_comment::SubmitCommentRequest,
            submit_post_request::SubmitPostRequest, update_comment_request::UpdateCommentRequest,
            update_post_request::UpdatePostRequest, upvote_comment_request::UpvoteCommentRequest,
            upvote_post_request::UpvotePostRequest,
        },
        forum::{
            moderation::{
                ForumReplyModerationActionRequest, ForumTopicModerationActionRequest,
                ModerateForumReplyRequest, ModerateForumTopicRequest,
            },
            replies::{CreateForumReplyRequest, UpdateForumReplyRequest},
            topics::{CreateForumTopicRequest, DeleteForumContentRequest, UpdateForumTopicRequest},
        },
        i18n::get_ui_text_bundle_request::GetUiTextBundleRequest,
        photography::{
            delete_photographs_request::DeletePhotographsRequest,
            submit_photograph_comment_request::SubmitPhotographCommentRequest,
            update_photograph_comment_request::UpdatePhotographCommentRequest,
            vote_photograph_request::VotePhotographRequest,
        },
        wasm_module::UpdateWasmModuleRequest,
    },
    responses::{
        admin::{
            authorization_response::{
                AuthorizationAuditCursorItem, AuthorizationAuditItem, AuthorizationAuditResponse,
                AuthorizationPermissionItem, AuthorizationPermissionsResponse,
                AuthorizationRoleItem, AuthorizationRolesResponse, AuthorizationUserItem,
                AuthorizationUsersResponse, RoleAssignmentResponse, RolePermissionChangeResponse,
                RolePermissionItem, RolePermissionsResponse,
            },
            media_cleanup_response::{
                ResolveMediaCleanupResponse, UnresolvedMediaCleanupItem,
                UnresolvedMediaCleanupResponse,
            },
            retention_notification_response::{
                RetentionNotificationStatusItem, RetentionNotificationStatusResponse,
                RetryRetentionNotificationResponse,
            },
            sync_i18n_cache_response::SyncI18nCacheResponse,
        },
        auth::{
            delete_account_response::DeleteAccountResponse,
            hard_purge_account_response::{HardPurgeAccountResponse, ProfileObjectCleanupFailure},
            is_superuser_response::IsSuperuserResponse,
            login_response::LoginResponse,
            logout_response::LogoutResponse,
            me_response::{MeResponse, UserInfo, UserProfilePicture},
            oidc_response::{OidcAuthorizationResponse, OidcLinkResponse, OidcStatusResponse},
            reset_password_request_response::ResetPasswordRequestResponse,
            reset_password_response::ResetPasswordResponse,
            signup_response::SignupResponse,
            update_profile_response::UpdateProfileResponse,
            verify_user_email_response::VerifyUserEmailResponse,
        },
        blog::{
            delete_comment_response::DeleteCommentResponse,
            delete_post_response::DeletePostResponse, get_posts::GetPostsResponse,
            read_post_response::ReadPostResponse, submit_post_response::SubmitPostResponse,
            vote_comment_response::VoteCommentResponse, vote_post_response::VotePostResponse,
        },
        forum::{
            common::{
                ForumAuthorResponse, ForumContentStateResponse, ForumNotificationKindResponse,
                ForumTopicAccessStateResponse,
            },
            moderation::{
                ForumModerationActionResponse, ForumModerationAuditCursorResponse,
                ForumModerationAuditItem, ForumModerationAuditListResponse,
                ForumModerationResponse,
            },
            notifications::{
                ForumNotificationCursorResponse, ForumNotificationListResponse,
                ForumNotificationReadResponse, ForumNotificationResponse,
            },
            topics::{
                ForumCapabilitiesResponse, ForumReplyCursorResponse, ForumReplyMutationResponse,
                ForumReplyResponse, ForumSubscriptionResponse, ForumTopicCursorResponse,
                ForumTopicDetailResponse, ForumTopicListResponse, ForumTopicMutationResponse,
                ForumTopicResponse,
            },
        },
        i18n::ui_text_bundle_response::UiTextBundleResponse,
        live_chat::{
            get_live_chat_messages_response::GetLiveChatMessagesResponse,
            live_chat_cache_stats_response::LiveChatCacheStatsResponse,
            live_chat_message_response::LiveChatMessageItem,
        },
        photography::{
            batch_status_response::{
                BatchItemStatus, BatchListResponse, BatchStatusResponse, BatchUploadItem,
                BatchUploadResponse,
            },
            delete_photograph_comment_response::DeletePhotographCommentResponse,
            delete_photographs_response::DeletePhotographsResponse,
            get_photograph_response::{GetPhotographsResponse, PaginationMeta, PhotographItem},
            read_photograph_response::ReadPhotographResponse,
            vote_photograph_response::VotePhotographResponse,
        },
        user::{
            profile_picture_history_response::{
                DeleteProfilePictureResponse, ProfilePictureHistoryItem,
                ProfilePictureHistoryResponse, SelectProfilePictureResponse,
            },
            public_user_info_response::PublicUserInfoResponse,
        },
        wasm_module::{GetWasmModulesResponse, WasmModuleItem},
    },
};
pub(crate) use crate::errors::code_error::CodeErrorResp;
pub(crate) use crate::features::accounts::api::{
    authorization_audit, authorization_mutations, authorization_queries, delete_account,
    hard_purge_account, is_superuser, login, logout, me, media_cleanup, oidc_callback, oidc_link,
    oidc_start, oidc_status, profile_picture_history, public_user, reset_password,
    reset_password_request, retention_notifications, signup, update_profile,
    upload_profile_picture, verify_user_email,
};
pub(crate) use crate::features::blog::api::{
    delete_comment, delete_post, get_posts, read_post, rescind_comment_vote, rescind_post_vote,
    search_posts, submit_comment, submit_post, update_comment, update_post, vote_comment,
    vote_post,
};
pub(crate) use crate::features::forum::api::{
    audit as forum_audit, capabilities as forum_capabilities, notifications as forum_notifications,
    replies as forum_replies, subscriptions as forum_subscriptions, topics as forum_topics,
};
pub(crate) use crate::features::geo::api::lookup as geo_lookup;
pub(crate) use crate::features::i18n::api::{get_ui_text_bundle, sync_i18n_cache};
pub(crate) use crate::features::live_chat::api::{cache_stats, get_messages};
pub(crate) use crate::features::photography::api::{
    batch_list, batch_status, batch_upload, delete_photograph_comment, delete_photographs,
    get_photographs, read_photograph, rescind_photograph_comment_vote, rescind_photograph_vote,
    submit_photograph_comment, update_photograph_comment, upload_photograph, vote_photograph,
    vote_photograph_comment,
};
pub(crate) use crate::features::reference_data::api::{
    get_countries, get_country, get_language, get_languages,
    get_subdivisions as get_subdivisions_for_country,
};
pub(crate) use crate::features::server_status::api::http as server_http;
pub(crate) use crate::features::visitor::api::visitor_board as forum_visitor_board;
pub(crate) use crate::features::wasm::api::{
    delete_module as delete_wasm_module, list_modules as get_wasm_modules,
    serve_bundle as serve_wasm, update_assets as update_wasm_module_assets,
    update_metadata as update_wasm_module, upload_module as upload_wasm_module,
};
pub(crate) use crate::features::{
    accounts::domain::retention_notifications::RetentionNotificationStage,
    blog::{
        api::search_posts::SearchPostsResponse,
        domain::{
            comment::{Comment, CommentResponse},
            post::{Post, PostInfo, PostInfoWithVote, UserBadgeInfo},
            tag::Tag,
            vote::VoteState,
        },
    },
    geo::domain::geo_ip::IpInfo,
    photography::domain::{
        batch::ProcessingStatus,
        photograph::{Photograph, PhotographContext},
        social::{PhotographComment, PhotographCommentResponse},
    },
    reference_data::{
        api::get_countries::GetCountriesResponse,
        domain::{
            country::{CountryAndSubdivisions, IsoCountry, IsoCountrySubdivision},
            currency::IsoCurrency,
            language::IsoLanguage,
        },
    },
    server_status::api::http::{RootHandlerResponse, ServerHealthcheckResponse},
    wasm::api::delete_module::DeleteWasmModuleResponse,
};
pub(crate) use crate::openapi_envelope::FrontendResponseEnvelope;
