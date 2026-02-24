use super::{PasswordChangeSuccess, UserMutationResult};
use crate::errors::{AuthError, DbError, ValidationErrorType};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::{Context, Result};
use models::{prelude::*, *};
use sea_orm::*;
use services::authentication::refresh_token::{cleanup_expired_tokens, revoke_all_refresh_tokens};
use services::verification_token::{validate_token, TokenKind};

pub(super) async fn reset_password(
    ctx: &Context<'_>,
    token: String,
    new_password: String,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let record = match validate_token(db, &token, TokenKind::PasswordReset).await {
        Ok(r) => r,
        Err(e) => return Ok(UserMutationResult::AuthError(AuthError { message: e.message })),
    };

    let temp_user = users::ActiveModel {
        id: ActiveValue::set(record.user_id),
        email: ActiveValue::set("placeholder@example.com".to_string()),
        password: ActiveValue::set(new_password.clone()),
        ..Default::default()
    };
    use services::validation::ActiveModelValidator;
    if let Err(e) = temp_user.validate() {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: e.to_string(),
        }));
    }

    let salt = SaltString::generate(&mut OsRng);
    let new_hash = match Argon2::default().hash_password(&new_password.into_bytes(), &salt) {
        Ok(h) => h.to_string(),
        Err(_) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Failed to hash password".to_string(),
            }))
        }
    };

    let mut user_active = users::ActiveModel {
        id: ActiveValue::set(record.user_id),
        ..Default::default()
    };
    user_active.password = ActiveValue::set(new_hash);
    user_active.updated_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
    if let Err(e) = Users::update(user_active).exec(db).await {
        return Ok(UserMutationResult::DbError(DbError { message: e.to_string() }));
    }

    let _ = revoke_all_refresh_tokens(db, record.user_id).await;
    let _ = cleanup_expired_tokens(db).await;

    tracing::info!(user_id = %record.user_id, "password reset");
    Ok(UserMutationResult::PasswordChangeSuccess(PasswordChangeSuccess {
        message: "Password reset successfully. Please sign in.".to_string(),
    }))
}
