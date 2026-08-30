//! Password-reset and email-verification token persistence.

use chrono::{DateTime, Utc};
use diesel::{ExpressionMethods, OptionalExtension, QueryDsl, SelectableHelper};
use diesel_async::{AsyncConnection, RunQueryDsl};
use uuid::Uuid;

use crate::{
    features::accounts::{
        domain::account::{
            EmailVerificationReceipt, EmailVerificationToken, PasswordResetReceipt,
            PasswordResetRequestReceipt, PasswordResetToken,
        },
        error::AccountError,
        repository::{
            account_repository::AccountRepository,
            records::{
                AccountRecord, EmailVerificationTokenRecord, NewPasswordResetTokenRecord,
                PasswordResetTokenRecord, UpdatePasswordRecord,
            },
        },
    },
    schema::{email_verification_tokens, password_reset_tokens, users},
};

impl AccountRepository {
    pub async fn issue_password_reset_token(
        &self,
        user_email: &str,
        token: Uuid,
        created_at: DateTime<Utc>,
        expires_at: DateTime<Utc>,
    ) -> Result<PasswordResetRequestReceipt, AccountError> {
        let mut connection = self.connection().await?;
        connection
            .transaction::<PasswordResetRequestReceipt, AccountError, _>(
                async move |connection| {
                    let account = users::table
                        .filter(users::user_email.eq(user_email))
                        .filter(users::user_deleted_at.is_null())
                        .select((users::user_id, users::user_email))
                        .for_update()
                        .first::<(Uuid, String)>(&mut *connection)
                        .await
                        .optional()?;
                    let (user_id, stored_email) =
                        account.ok_or(AccountError::AccountNotFound)?;
                    diesel::insert_into(password_reset_tokens::table)
                        .values(NewPasswordResetTokenRecord {
                            user_id,
                            password_reset_token: token,
                            password_reset_token_expires_at: expires_at,
                            password_reset_token_created_at: created_at,
                        })
                        .execute(&mut *connection)
                        .await?;
                    Ok(PasswordResetRequestReceipt {
                        user_email: stored_email,
                        token,
                        verify_by: expires_at,
                    })
                },
            )
            .await
    }

    pub async fn password_reset_token(
        &self,
        token: Uuid,
    ) -> Result<Option<PasswordResetToken>, AccountError> {
        let mut connection = self.connection().await?;
        password_reset_tokens::table
            .filter(password_reset_tokens::password_reset_token.eq(token))
            .select(PasswordResetTokenRecord::as_select())
            .first::<PasswordResetTokenRecord>(&mut connection)
            .await
            .optional()
            .map(|record| record.map(Into::into))
            .map_err(AccountError::Query)
    }

    pub async fn consume_password_reset_token(
        &self,
        token: &PasswordResetToken,
        consumed_at: DateTime<Utc>,
        password_hash: &str,
    ) -> Result<PasswordResetReceipt, AccountError> {
        let mut connection = self.connection().await?;
        let update = UpdatePasswordRecord {
            user_password_hash: password_hash,
            user_updated_at: consumed_at,
        };
        let transaction_result = connection
            .transaction::<AccountRecord, diesel::result::Error, _>(async |connection| {
                let consumed = diesel::update(
                    password_reset_tokens::table
                        .filter(password_reset_tokens::password_reset_token_id.eq(token.token_id))
                        .filter(password_reset_tokens::password_reset_token_used_at.is_null()),
                )
                .set(password_reset_tokens::password_reset_token_used_at.eq(consumed_at))
                .execute(&mut *connection)
                .await?;

                if consumed != 1 {
                    return Err(diesel::result::Error::RollbackTransaction);
                }

                diesel::update(
                    users::table
                        .filter(users::user_id.eq(token.user_id))
                        .filter(users::user_deleted_at.is_null()),
                )
                .set(&update)
                .returning(AccountRecord::as_returning())
                .get_result(&mut *connection)
                .await
            })
            .await;

        match transaction_result {
            Ok(account) => Ok(account.into_password_reset_receipt()),
            Err(diesel::result::Error::RollbackTransaction) => {
                Err(AccountError::TokenAlreadyConsumed)
            }
            Err(error) => Err(AccountError::Mutation(error)),
        }
    }

    pub async fn email_verification_token(
        &self,
        token: Uuid,
    ) -> Result<Option<EmailVerificationToken>, AccountError> {
        let mut connection = self.connection().await?;
        email_verification_tokens::table
            .filter(email_verification_tokens::email_verification_token.eq(token))
            .select(EmailVerificationTokenRecord::as_select())
            .first::<EmailVerificationTokenRecord>(&mut connection)
            .await
            .optional()
            .map(|record| record.map(Into::into))
            .map_err(AccountError::Query)
    }

    pub async fn consume_email_verification_token(
        &self,
        token: &EmailVerificationToken,
        verified_at: DateTime<Utc>,
    ) -> Result<EmailVerificationReceipt, AccountError> {
        let mut connection = self.connection().await?;
        let transaction_result = connection
            .transaction::<String, diesel::result::Error, _>(async |connection| {
                let consumed = diesel::update(
                    email_verification_tokens::table
                        .filter(
                            email_verification_tokens::email_verification_token_id.eq(token.token_id),
                        )
                        .filter(email_verification_tokens::email_verification_token_used_at.is_null()),
                )
                .set(email_verification_tokens::email_verification_token_used_at.eq(verified_at))
                .execute(&mut *connection)
                .await?;

                if consumed != 1 {
                    return Err(diesel::result::Error::RollbackTransaction);
                }

                diesel::update(
                    users::table
                        .filter(users::user_id.eq(token.user_id))
                        .filter(users::user_is_email_verified.eq(false))
                        .filter(users::user_deleted_at.is_null()),
                )
                .set((
                    users::user_is_email_verified.eq(true),
                    users::user_updated_at.eq(verified_at),
                ))
                .returning(users::user_email)
                .get_result::<String>(&mut *connection)
                .await
            })
            .await;

        match transaction_result {
            Ok(user_email) => Ok(EmailVerificationReceipt {
                user_id: token.user_id,
                user_email,
                verified_at,
            }),
            Err(diesel::result::Error::RollbackTransaction) => {
                Err(AccountError::TokenAlreadyConsumed)
            }
            Err(diesel::result::Error::NotFound) => Err(AccountError::EmailAlreadyVerified),
            Err(error) => Err(AccountError::Mutation(error)),
        }
    }
}
