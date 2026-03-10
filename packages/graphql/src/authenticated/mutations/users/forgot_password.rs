use crate::errors::DbError;
use async_graphql::{Context, Object, Result, SimpleObject, Union};
use models::{prelude::*, *};
use sea_orm::*;
use services::email::EmailService;
use services::verification_token::{create_token, TokenKind};

#[derive(SimpleObject)]
pub struct PasswordResetSuccess {
    pub message: String,
}

#[derive(Union)]
pub enum ForgotPasswordResult {
    PasswordResetSuccess(PasswordResetSuccess),
    DbError(DbError),
}

#[derive(Default)]
pub struct ForgotPasswordMutation;

#[Object]
impl ForgotPasswordMutation {
    async fn forgot_password(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> Result<ForgotPasswordResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let ok = ForgotPasswordResult::PasswordResetSuccess(PasswordResetSuccess {
            message: "If that email exists, a reset link was sent".to_string(),
        });

        let user = match Users::find()
            .filter(users::Column::Email.eq(&email))
            .one(db)
            .await
        {
            Ok(Some(u)) => u,
            Ok(None) => return Ok(ok),
            Err(e) => return Ok(ForgotPasswordResult::DbError(DbError { message: e.to_string() })),
        };

        if let Ok(email_service) = ctx.data::<EmailService>() {
            match create_token(db, user.id, TokenKind::PasswordReset, 3600).await {
                Ok(raw_token) => {
                    if let Err(e) = email_service.send_password_reset(&email, &raw_token).await {
                        tracing::warn!(user_id = %user.id, error = %e, "failed to send password reset email");
                    }
                }
                Err(e) => tracing::warn!(user_id = %user.id, error = %e.message, "failed to create reset token"),
            }
        }

        Ok(ok)
    }
}
