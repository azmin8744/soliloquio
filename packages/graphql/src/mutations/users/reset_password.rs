use super::PasswordChangeSuccess;
use crate::errors::{AuthError, DbError};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::{Context, Object, Result, Union};
use repositories::UserRepository;
use sea_orm::*;
use services::authentication::refresh_token::{cleanup_expired_tokens, revoke_all_refresh_tokens};
use services::verification_token::{validate_token, TokenKind};

#[derive(Union)]
pub enum ResetPasswordResult {
    PasswordChangeSuccess(PasswordChangeSuccess),
    AuthError(AuthError),
    DbError(DbError),
}

#[derive(Default)]
pub struct ResetPasswordMutation;

#[Object]
impl ResetPasswordMutation {
    async fn reset_password(
        &self,
        ctx: &Context<'_>,
        token: String,
        new_password: String,
    ) -> Result<ResetPasswordResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let record = match validate_token(db, &token, TokenKind::PasswordReset).await {
            Ok(r) => r,
            Err(e) => return Ok(ResetPasswordResult::AuthError(AuthError { message: e.message })),
        };

        let salt = SaltString::generate(&mut OsRng);
        let new_hash = match Argon2::default().hash_password(&new_password.into_bytes(), &salt) {
            Ok(h) => h.to_string(),
            Err(_) => {
                return Ok(ResetPasswordResult::AuthError(AuthError {
                    message: "Failed to hash password".to_string(),
                }))
            }
        };

        if let Err(e) = UserRepository::update_password(db, record.user_id, new_hash).await {
            return Ok(ResetPasswordResult::DbError(DbError { message: e.to_string() }));
        }

        let _ = revoke_all_refresh_tokens(db, record.user_id).await;
        let _ = cleanup_expired_tokens(db).await;

        tracing::info!(user_id = %record.user_id, "password reset");
        Ok(ResetPasswordResult::PasswordChangeSuccess(PasswordChangeSuccess {
            message: "Password reset successfully. Please sign in.".to_string(),
        }))
    }
}
