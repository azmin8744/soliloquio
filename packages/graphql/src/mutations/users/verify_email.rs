use super::{EmailVerifySuccess, UserMutationResult};
use crate::errors::{AuthError, DbError};
use async_graphql::{Context, Result};
use models::{prelude::*, *};
use sea_orm::*;
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

    let mut user_active = users::ActiveModel {
        id: ActiveValue::set(record.user_id),
        ..Default::default()
    };
    user_active.email_verified_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
    if let Err(e) = Users::update(user_active).exec(db).await {
        return Ok(UserMutationResult::DbError(DbError { message: e.to_string() }));
    }

    let _ = cleanup_expired(db).await;

    tracing::info!(user_id = %record.user_id, "email verified");
    Ok(UserMutationResult::EmailVerifySuccess(EmailVerifySuccess {
        message: "Email verified successfully".to_string(),
    }))
}
