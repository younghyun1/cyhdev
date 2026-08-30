//! OpenID Connect identity persistence and account-resolution queries.

use chrono::Utc;
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper, dsl::exists};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;
use zeroize::Zeroizing;

use crate::{
    features::accounts::{
        domain::{
            account::SessionPrincipal,
            oidc::{OidcAccount, OidcIdentityClaims, OidcUnlinkCandidate},
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            oidc_records::{NewOidcIdentityRecord, OidcAccountRecord, OidcLoginRecord},
        },
    },
    schema::{account_oidc_identities, users},
};

const ISSUER_SUBJECT_UNIQUE: &str = "account_oidc_identities_issuer_subject_unique";

impl AccountRepository {
    pub async fn oidc_is_linked(&self, user_id: Uuid, issuer: &str) -> Result<bool, AccountError> {
        let mut connection = self.connection().await?;
        diesel::select(exists(
            account_oidc_identities::table
                .filter(account_oidc_identities::account_oidc_identity_user_id.eq(user_id))
                .filter(account_oidc_identities::account_oidc_identity_issuer.eq(issuer)),
        ))
        .get_result(&mut connection)
        .await
        .map_err(AccountError::Query)
    }

    /// Resolves login authority only by the provider's stable issuer and subject pair.
    pub async fn oidc_account_for_login(
        &self,
        identity: &OidcIdentityClaims,
    ) -> Result<Option<OidcAccount>, AccountError> {
        let mut connection = self.connection().await?;
        let now = Utc::now();
        connection
            .transaction::<Option<OidcAccount>, AccountError, _>(async move |connection| {
                let record = account_oidc_identities::table
                    .inner_join(users::table)
                    .filter(
                        account_oidc_identities::account_oidc_identity_issuer.eq(&identity.issuer),
                    )
                    .filter(
                        account_oidc_identities::account_oidc_identity_subject.eq(&identity.subject),
                    )
                    .filter(users::user_deleted_at.is_null())
                    .filter(users::user_is_email_verified.eq(true))
                    .select((
                        account_oidc_identities::account_oidc_identity_id,
                        users::user_id,
                        users::user_name,
                        users::user_is_email_verified,
                        users::user_country,
                        users::user_language,
                    ))
                    .for_update()
                    .first::<(Uuid, Uuid, String, bool, i32, i32)>(&mut *connection)
                    .await
                    .optional()?
                    .map(OidcLoginRecord::from);
                let record = match record {
                    Some(record) => record,
                    None => return Ok(None),
                };
                diesel::update(
                    account_oidc_identities::table.filter(
                        account_oidc_identities::account_oidc_identity_id.eq(record.oidc_identity_id),
                    ),
                )
                .set((
                    account_oidc_identities::account_oidc_identity_provider_email
                        .eq(&identity.provider_email),
                    account_oidc_identities::account_oidc_identity_updated_at.eq(now),
                    account_oidc_identities::account_oidc_identity_last_authenticated_at.eq(now),
                ))
                .execute(&mut *connection)
                .await?;
                Ok(Some(record.into()))
            })
            .await
    }

    pub async fn link_oidc_identity(
        &self,
        user_id: Uuid,
        identity: &OidcIdentityClaims,
    ) -> Result<SessionPrincipal, AccountError> {
        let mut connection = self.connection().await?;
        let now = Utc::now();
        connection
            .transaction::<SessionPrincipal, AccountError, _>(async move |connection| {
                let account = lock_verified_account(connection, user_id).await?;
                let existing = account_oidc_identities::table
                    .filter(account_oidc_identities::account_oidc_identity_user_id.eq(user_id))
                    .filter(
                        account_oidc_identities::account_oidc_identity_issuer.eq(&identity.issuer),
                    )
                    .select((
                        account_oidc_identities::account_oidc_identity_id,
                        account_oidc_identities::account_oidc_identity_subject,
                    ))
                    .first::<(Uuid, String)>(&mut *connection)
                    .await
                    .optional()?;
                if let Some((identity_id, subject)) = existing {
                    if subject != identity.subject {
                        return Err(AccountError::OidcProviderAlreadyLinked);
                    }
                    diesel::update(
                        account_oidc_identities::table.filter(
                            account_oidc_identities::account_oidc_identity_id.eq(identity_id),
                        ),
                    )
                    .set((
                        account_oidc_identities::account_oidc_identity_provider_email
                            .eq(&identity.provider_email),
                        account_oidc_identities::account_oidc_identity_updated_at.eq(now),
                    ))
                    .execute(&mut *connection)
                    .await?;
                    return Ok(account.into());
                }

                let result = diesel::insert_into(account_oidc_identities::table)
                    .values(NewOidcIdentityRecord {
                        user_id,
                        issuer: &identity.issuer,
                        subject: &identity.subject,
                        provider_email: &identity.provider_email,
                        created_at: now,
                        updated_at: now,
                    })
                    .execute(&mut *connection)
                    .await;
                match result {
                    Ok(_) => Ok(account.into()),
                    Err(error) => Err(classify_identity_insert(error)),
                }
            })
            .await
    }

    pub async fn oidc_unlink_candidate(
        &self,
        user_id: Uuid,
        issuer: &str,
    ) -> Result<OidcUnlinkCandidate, AccountError> {
        let mut connection = self.connection().await?;
        let password_hash = users::table
            .inner_join(account_oidc_identities::table)
            .filter(users::user_id.eq(user_id))
            .filter(users::user_deleted_at.is_null())
            .filter(users::user_is_email_verified.eq(true))
            .filter(account_oidc_identities::account_oidc_identity_issuer.eq(issuer))
            .select(users::user_password_hash)
            .first::<String>(&mut connection)
            .await
            .optional()
            .map_err(AccountError::Query)?
            .ok_or(AccountError::OidcIdentityNotFound)?;
        if password_hash.is_empty() {
            return Err(AccountError::OidcAnotherLoginRequired);
        }
        Ok(OidcUnlinkCandidate {
            password_hash: Zeroizing::new(password_hash),
        })
    }

    pub async fn unlink_oidc_identity(
        &self,
        user_id: Uuid,
        issuer: &str,
        expected_password_hash: &str,
    ) -> Result<SessionPrincipal, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<SessionPrincipal, AccountError, _>(async move |connection| {
                let account = lock_verified_account(connection, user_id).await?;
                if account.user_password_hash.is_empty() {
                    return Err(AccountError::OidcAnotherLoginRequired);
                }
                if account.user_password_hash != expected_password_hash {
                    return Err(AccountError::AccountChanged);
                }
                let deleted = diesel::delete(
                    account_oidc_identities::table
                        .filter(account_oidc_identities::account_oidc_identity_user_id.eq(user_id))
                        .filter(account_oidc_identities::account_oidc_identity_issuer.eq(issuer)),
                )
                .execute(&mut *connection)
                .await?;
                if deleted == 0 {
                    return Err(AccountError::OidcIdentityNotFound);
                }
                Ok(account.into())
            })
            .await
    }
}

async fn lock_verified_account(
    connection: &mut diesel_async::AsyncPgConnection,
    user_id: Uuid,
) -> Result<OidcAccountRecord, AccountError> {
    let account = users::table
        .filter(users::user_id.eq(user_id))
        .filter(users::user_deleted_at.is_null())
        .filter(users::user_is_email_verified.eq(true))
        .select(OidcAccountRecord::as_select())
        .for_update()
        .first::<OidcAccountRecord>(&mut *connection)
        .await
        .optional()?;
    account.ok_or(AccountError::OidcIdentityNotLinked)
}

fn classify_identity_insert(error: diesel::result::Error) -> AccountError {
    match &error {
        diesel::result::Error::DatabaseError(diesel::result::DatabaseErrorKind::UniqueViolation, info)
            if info.constraint_name() == Some(ISSUER_SUBJECT_UNIQUE) =>
        {
            AccountError::OidcIdentityConflict(error)
        }
        _ => AccountError::Mutation(error),
    }
}
