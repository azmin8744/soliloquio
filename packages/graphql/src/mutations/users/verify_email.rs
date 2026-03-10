use crate::errors::{AuthError, DbError};
use async_graphql::{Context, Object, Result, SimpleObject, Union};
use repositories::UserRepository;
use sea_orm::DatabaseConnection;
use services::verification_token::{cleanup_expired, validate_token, TokenKind};

#[derive(SimpleObject)]
pub struct EmailVerifySuccess {
    pub message: String,
}

#[derive(Union)]
pub enum VerifyEmailResult {
    EmailVerifySuccess(EmailVerifySuccess),
    AuthError(AuthError),
    DbError(DbError),
}

#[derive(Default)]
pub struct VerifyEmailMutation;

#[Object]
impl VerifyEmailMutation {
    async fn verify_email(&self, ctx: &Context<'_>, token: String) -> Result<VerifyEmailResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let record = match validate_token(db, &token, TokenKind::EmailVerification).await {
            Ok(r) => r,
            Err(e) => return Ok(VerifyEmailResult::AuthError(AuthError { message: e.message })),
        };

        if let Err(e) = UserRepository::verify_email(db, record.user_id).await {
            return Ok(VerifyEmailResult::DbError(DbError { message: e }));
        }

        let _ = cleanup_expired(db).await;

        tracing::info!(user_id = %record.user_id, "email verified");
        Ok(VerifyEmailResult::EmailVerifySuccess(EmailVerifySuccess {
            message: "Email verified successfully".to_string(),
        }))
    }
}
