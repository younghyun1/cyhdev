//! PostgreSQL enum adapters kept outside the persistence-independent domain.

use std::io::Write;

use diesel::deserialize::{FromSql, Result as DeserializeResult};
use diesel::pg::{Pg, PgValue};
use diesel::query_builder::QueryId;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{AsExpression, FromSqlRow};

use crate::{
    features::accounts::domain::{
        authorization::AuthorizationAuditKind, retention_notifications::RetentionNotificationStage,
    },
    schema::sql_types::{
        AccountRetentionNotificationStage as RetentionStageSql,
        AuthorizationAuditKind as AuthorizationAuditKindSql,
    },
};

impl QueryId for AuthorizationAuditKindSql {
    type QueryId = AuthorizationAuditKindSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}

impl QueryId for RetentionStageSql {
    type QueryId = RetentionStageSql;
    const HAS_STATIC_QUERY_ID: bool = true;
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = AuthorizationAuditKindSql)]
pub(super) struct StoredAuthorizationAuditKind(AuthorizationAuditKind);

impl StoredAuthorizationAuditKind {
    pub(super) const fn into_domain(self) -> AuthorizationAuditKind {
        self.0
    }
}

impl From<AuthorizationAuditKind> for StoredAuthorizationAuditKind {
    fn from(value: AuthorizationAuditKind) -> Self {
        Self(value)
    }
}

impl ToSql<AuthorizationAuditKindSql, Pg> for StoredAuthorizationAuditKind {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        out.write_all(self.0.as_str().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<AuthorizationAuditKindSql, Pg> for StoredAuthorizationAuditKind {
    fn from_sql(bytes: PgValue<'_>) -> DeserializeResult<Self> {
        let value = match bytes.as_bytes() {
            b"user_role_assigned" => AuthorizationAuditKind::UserRoleAssigned,
            b"role_permission_granted" => AuthorizationAuditKind::RolePermissionGranted,
            b"role_permission_revoked" => AuthorizationAuditKind::RolePermissionRevoked,
            _ => return Err("unrecognized authorization_audit_kind value".into()),
        };
        Ok(Self(value))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, AsExpression, FromSqlRow)]
#[diesel(sql_type = RetentionStageSql)]
pub(super) struct StoredRetentionNotificationStage(RetentionNotificationStage);

impl StoredRetentionNotificationStage {
    pub(super) const fn into_domain(self) -> RetentionNotificationStage {
        self.0
    }
}

impl From<RetentionNotificationStage> for StoredRetentionNotificationStage {
    fn from(value: RetentionNotificationStage) -> Self {
        Self(value)
    }
}

impl ToSql<RetentionStageSql, Pg> for StoredRetentionNotificationStage {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> diesel::serialize::Result {
        out.write_all(self.0.as_str().as_bytes())?;
        Ok(IsNull::No)
    }
}

impl FromSql<RetentionStageSql, Pg> for StoredRetentionNotificationStage {
    fn from_sql(bytes: PgValue<'_>) -> DeserializeResult<Self> {
        let value = match bytes.as_bytes() {
            b"seven_days_before_purge" => RetentionNotificationStage::SevenDaysBeforePurge,
            b"one_day_before_purge" => RetentionNotificationStage::OneDayBeforePurge,
            _ => return Err("unrecognized account_retention_notification_stage value".into()),
        };
        Ok(Self(value))
    }
}
