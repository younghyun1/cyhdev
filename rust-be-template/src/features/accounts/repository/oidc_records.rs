//! Diesel records private to OpenID Connect identity persistence.

use chrono::{DateTime, Utc};
use uuid::Uuid;

use crate::{
    features::accounts::domain::{account::SessionPrincipal, oidc::OidcAccount},
    schema::{account_oidc_identities, users},
};

pub(super) struct OidcLoginRecord {
    pub(super) oidc_identity_id: Uuid,
    user_id: Uuid,
    user_name: String,
    is_email_verified: bool,
    country: i32,
    language: i32,
}

impl From<(Uuid, Uuid, String, bool, i32, i32)> for OidcLoginRecord {
    fn from(record: (Uuid, Uuid, String, bool, i32, i32)) -> Self {
        Self {
            oidc_identity_id: record.0,
            user_id: record.1,
            user_name: record.2,
            is_email_verified: record.3,
            country: record.4,
            language: record.5,
        }
    }
}

impl From<OidcLoginRecord> for OidcAccount {
    fn from(record: OidcLoginRecord) -> Self {
        Self {
            user_id: record.user_id,
            user_name: record.user_name,
            is_email_verified: record.is_email_verified,
            country: record.country,
            language: record.language,
        }
    }
}

#[derive(diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = users)]
#[diesel(check_for_backend(diesel::pg::Pg))]
pub(super) struct OidcAccountRecord {
    user_id: Uuid,
    user_name: String,
    pub(super) user_password_hash: String,
    user_is_email_verified: bool,
    user_country: i32,
    user_language: i32,
}

impl From<OidcAccountRecord> for SessionPrincipal {
    fn from(record: OidcAccountRecord) -> Self {
        Self {
            user_id: record.user_id,
            user_name: record.user_name,
            is_email_verified: record.user_is_email_verified,
            country: record.user_country,
            language: record.user_language,
        }
    }
}

#[derive(diesel::Insertable)]
#[diesel(table_name = account_oidc_identities)]
pub(super) struct NewOidcIdentityRecord<'a> {
    #[diesel(column_name = account_oidc_identity_user_id)]
    pub(super) user_id: Uuid,
    #[diesel(column_name = account_oidc_identity_issuer)]
    pub(super) issuer: &'a str,
    #[diesel(column_name = account_oidc_identity_subject)]
    pub(super) subject: &'a str,
    #[diesel(column_name = account_oidc_identity_provider_email)]
    pub(super) provider_email: &'a str,
    #[diesel(column_name = account_oidc_identity_created_at)]
    pub(super) created_at: DateTime<Utc>,
    #[diesel(column_name = account_oidc_identity_updated_at)]
    pub(super) updated_at: DateTime<Utc>,
}
