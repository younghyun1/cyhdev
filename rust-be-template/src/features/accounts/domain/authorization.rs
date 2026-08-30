//! Validated role-administration commands and authority records.

use chrono::{DateTime, Utc};
use nutype::nutype;
use uuid::Uuid;

use super::role::RoleType;

pub const DEFAULT_AUTHORIZATION_PAGE_SIZE: u16 = 50;
pub const MAX_AUTHORIZATION_CATALOG_ROWS: i64 = 256;

#[nutype(
    sanitize(trim),
    validate(len_char_min = 8, len_char_max = 500),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Display, TryFrom)
)]
pub struct AuthorizationReason(String);

#[nutype(
    validate(
        len_char_min = 3,
        len_char_max = 64,
        predicate = is_valid_permission_name
    ),
    derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash, AsRef, Display, TryFrom)
)]
pub struct PermissionName(String);

#[nutype(
    sanitize(trim),
    validate(not_empty, len_char_max = 100),
    derive(Debug, Clone, PartialEq, Eq, AsRef, Display, TryFrom)
)]
pub struct AuthorizationSearch(String);

#[nutype(
    validate(greater_or_equal = 1, less_or_equal = 100),
    derive(Debug, Clone, Copy, PartialEq, Eq, Into, TryFrom)
)]
pub struct AuthorizationPageSize(u16);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuthorizationAuditKind {
    UserRoleAssigned,
    RolePermissionGranted,
    RolePermissionRevoked,
}

impl AuthorizationAuditKind {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::UserRoleAssigned => "user_role_assigned",
            Self::RolePermissionGranted => "role_permission_granted",
            Self::RolePermissionRevoked => "role_permission_revoked",
        }
    }

}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationUser {
    pub user_id: Uuid,
    pub user_name: String,
    pub user_email: String,
    pub role_type: RoleType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleDefinition {
    pub role_type: RoleType,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PermissionDefinition {
    pub permission_id: Uuid,
    pub permission_name: PermissionName,
    pub description: Option<String>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePermissionBinding {
    pub role_permission_id: Uuid,
    pub role_type: RoleType,
    pub permission_id: Uuid,
    pub permission_name: PermissionName,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationAuditEvent {
    pub audit_event_id: Uuid,
    pub actor_user_id: Uuid,
    /// Current display name resolved from the permanent user row at read time.
    pub actor_display_name: String,
    pub kind: AuthorizationAuditKind,
    pub target_user_id: Option<Uuid>,
    /// Current display name resolved from the permanent user row at read time.
    pub target_display_name: Option<String>,
    pub role_id: Uuid,
    pub role_name: String,
    pub permission_id: Option<Uuid>,
    pub permission_name: Option<PermissionName>,
    pub old_value: String,
    pub new_value: String,
    pub reason: String,
    pub request_id: Option<Uuid>,
    pub created_at: DateTime<Utc>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationPage<T> {
    pub items: Vec<T>,
    pub next_cursor: Option<Uuid>,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AuthorizationAuditCursor {
    pub created_at: DateTime<Utc>,
    pub audit_event_id: Uuid,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AuthorizationAuditPage {
    pub items: Vec<AuthorizationAuditEvent>,
    pub next_cursor: Option<AuthorizationAuditCursor>,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RoleAssignmentReceipt {
    pub audit_event_id: Uuid,
    pub user_id: Uuid,
    pub previous_role: RoleType,
    pub role_type: RoleType,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RolePermissionReceipt {
    pub audit_event_id: Uuid,
    pub role_type: RoleType,
    pub permission_id: Uuid,
    pub permission_name: PermissionName,
    pub enabled: bool,
}

fn is_valid_permission_name(value: &str) -> bool {
    let mut segment_count = 0usize;
    for segment in value.split('.') {
        segment_count += 1;
        let mut bytes = segment.bytes();
        let first_is_valid = bytes
            .next()
            .is_some_and(|byte| byte.is_ascii_lowercase());
        if !first_is_valid
            || !bytes.all(|byte| {
                byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'_'
            })
        {
            return false;
        }
    }
    segment_count >= 2
}

#[cfg(test)]
mod tests {
    use super::{
        AuthorizationPageSize, AuthorizationReason, DEFAULT_AUTHORIZATION_PAGE_SIZE,
        PermissionName,
    };

    #[test]
    fn permission_names_require_lowercase_namespaced_segments() {
        assert!(PermissionName::try_new("authorization.roles_manage").is_ok());
        assert!(PermissionName::try_new("Authorization.roles").is_err());
        assert!(PermissionName::try_new("single").is_err());
        assert!(PermissionName::try_new("authorization..roles").is_err());
    }

    #[test]
    fn audit_reasons_are_trimmed_and_character_bounded() {
        let reason = AuthorizationReason::try_new("  grant support access  ");
        assert!(matches!(reason, Ok(value) if value.as_ref() == "grant support access"));
        assert!(AuthorizationReason::try_new("short").is_err());
        assert!(AuthorizationReason::try_new("가".repeat(501)).is_err());
    }

    #[test]
    fn page_size_default_stays_inside_public_bounds() {
        assert!(AuthorizationPageSize::try_new(DEFAULT_AUTHORIZATION_PAGE_SIZE).is_ok());
        assert!(AuthorizationPageSize::try_new(0).is_err());
        assert!(AuthorizationPageSize::try_new(101).is_err());
    }
}
