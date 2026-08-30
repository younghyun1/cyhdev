//! Keyset-paginated authorization audit reads with privacy-safe user resolution.

use std::collections::HashMap;

use diesel::{BoolExpressionMethods, ExpressionMethods, QueryDsl, SelectableHelper};
use diesel_async::RunQueryDsl;
use uuid::Uuid;

use crate::{
    features::accounts::{
        authorization_error::AuthorizationError,
        domain::authorization::{
            AuthorizationAuditCursor, AuthorizationAuditEvent, AuthorizationAuditPage,
            AuthorizationPageSize, PermissionName,
        },
        repository::{
            account_repository::AccountRepository,
            authorization_guard::ensure_current_younghyun,
            authorization_records::AuthorizationAuditEventRow,
        },
    },
    schema::{authorization_audit_events, users},
};

impl AccountRepository {
    pub async fn authorization_audit_events(
        &self,
        actor_user_id: Uuid,
        before: Option<AuthorizationAuditCursor>,
        page_size: AuthorizationPageSize,
    ) -> Result<AuthorizationAuditPage, AuthorizationError> {
        let mut connection = self.connection().await?;
        ensure_current_younghyun(&mut connection, actor_user_id).await?;
        let mut query = authorization_audit_events::table
            .order((
                authorization_audit_events::authorization_audit_event_created_at.desc(),
                authorization_audit_events::authorization_audit_event_id.desc(),
            ))
            .into_boxed();
        if let Some(before) = before {
            query = query.filter(
                authorization_audit_events::authorization_audit_event_created_at
                    .lt(before.created_at)
                    .or(authorization_audit_events::authorization_audit_event_created_at
                        .eq(before.created_at)
                        .and(
                            authorization_audit_events::authorization_audit_event_id
                                .lt(before.audit_event_id),
                        )),
            );
        }
        let mut rows = query
            .limit(i64::from(page_size.into_inner()) + 1)
            .select(AuthorizationAuditEventRow::as_select())
            .load::<AuthorizationAuditEventRow>(&mut connection)
            .await?;
        let next_cursor = take_audit_cursor(&mut rows, page_size);
        let names = load_audit_user_names(&mut connection, &rows).await?;
        let items = rows
            .into_iter()
            .map(|row| map_audit_event(row, &names))
            .collect::<Result<Vec<_>, AuthorizationError>>()?;
        Ok(AuthorizationAuditPage { items, next_cursor })
    }
}

fn map_audit_event(
    row: AuthorizationAuditEventRow,
    user_names: &HashMap<Uuid, String>,
) -> Result<AuthorizationAuditEvent, AuthorizationError> {
    let permission_name = row
        .authorization_audit_event_permission_name
        .map(PermissionName::try_new)
        .transpose()
        .map_err(|_| AuthorizationError::InvalidPermissionName)?;
    Ok(AuthorizationAuditEvent {
        audit_event_id: row.authorization_audit_event_id,
        actor_user_id: row.authorization_audit_event_actor_user_id,
        actor_display_name: resolved_user_name(
            user_names,
            row.authorization_audit_event_actor_user_id,
        )?,
        kind: row.authorization_audit_event_kind,
        target_user_id: row.authorization_audit_event_target_user_id,
        target_display_name: row
            .authorization_audit_event_target_user_id
            .map(|user_id| resolved_user_name(user_names, user_id))
            .transpose()?,
        role_id: row.authorization_audit_event_role_id,
        role_name: row.authorization_audit_event_role_name,
        permission_id: row.authorization_audit_event_permission_id,
        permission_name,
        old_value: row.authorization_audit_event_old_value,
        new_value: row.authorization_audit_event_new_value,
        reason: row.authorization_audit_event_reason,
        request_id: row.authorization_audit_event_request_id,
        created_at: row.authorization_audit_event_created_at,
    })
}

fn resolved_user_name(
    user_names: &HashMap<Uuid, String>,
    user_id: Uuid,
) -> Result<String, AuthorizationError> {
    user_names
        .get(&user_id)
        .cloned()
        .ok_or(AuthorizationError::AuditUserMissing)
}

fn take_audit_cursor(
    rows: &mut Vec<AuthorizationAuditEventRow>,
    page_size: AuthorizationPageSize,
) -> Option<AuthorizationAuditCursor> {
    if rows.len() <= usize::from(page_size.into_inner()) {
        return None;
    }
    rows.pop();
    rows.last().map(|row| AuthorizationAuditCursor {
        created_at: row.authorization_audit_event_created_at,
        audit_event_id: row.authorization_audit_event_id,
    })
}

async fn load_audit_user_names(
    connection: &mut diesel_async::AsyncPgConnection,
    rows: &[AuthorizationAuditEventRow],
) -> Result<HashMap<Uuid, String>, AuthorizationError> {
    let mut user_ids = Vec::with_capacity(rows.len().saturating_mul(2));
    for row in rows {
        user_ids.push(row.authorization_audit_event_actor_user_id);
        if let Some(target_user_id) = row.authorization_audit_event_target_user_id {
            user_ids.push(target_user_id);
        }
    }
    user_ids.sort_unstable();
    user_ids.dedup();
    if user_ids.is_empty() {
        return Ok(HashMap::new());
    }
    users::table
        .filter(users::user_id.eq_any(user_ids))
        .select((users::user_id, users::user_name))
        .load::<(Uuid, String)>(&mut *connection)
        .await
        .map(|rows| rows.into_iter().collect())
        .map_err(AuthorizationError::Query)
}
