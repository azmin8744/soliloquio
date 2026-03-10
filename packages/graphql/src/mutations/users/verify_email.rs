use super::{EmailVerifySuccess, UserMutationResult};
use crate::errors::{AuthError, DbError};
use async_graphql::{Context, Result};
use repositories::UserRepository;
use sea_orm::DatabaseConnection;
use services::verification_token::{cleanup_expired, validate_token, TokenKind};

pub(super) async fn verify_email(
    ctx: &Context<'_>,
    token: String,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let record = match validate_token(db, &token, TokenKind::EmailVerification).await {
        Ok(r) => r,
        Err(e) => return Ok(UserMutationResult::AuthError(AuthError { message: e.message })),
    };

    if let Err(e) = UserRepository::verify_email(db, record.user_id).await {
        return Ok(UserMutationResult::DbError(DbError { message: e }));
    }

    let _ = cleanup_expired(db).await;

    tracing::info!(user_id = %record.user_id, "email verified");
    Ok(UserMutationResult::EmailVerifySuccess(EmailVerifySuccess {
        message: "Email verified successfully".to_string(),
    }))
}
