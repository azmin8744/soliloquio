use super::{EmailVerifySuccess, UserMutation, UserMutationResult};
use crate::errors::AuthError;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result};
use sea_orm::DatabaseConnection;
use services::email::EmailService;
use services::verification_token::{create_token, TokenKind};

pub(super) async fn resend_verification_email(
    mutation: &UserMutation,
    ctx: &Context<'_>,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let user = match mutation.require_authenticate_as_user(ctx).await {
        Ok(u) => u,
        Err(e) => return Ok(UserMutationResult::AuthError(AuthError { message: e.to_string() })),
    };

    if user.email_verified_at.is_some() {
        return Ok(UserMutationResult::EmailVerifySuccess(EmailVerifySuccess {
            message: "Email is already verified".to_string(),
        }));
    }

    if let Ok(email_service) = ctx.data::<EmailService>() {
        match create_token(db, user.id, TokenKind::EmailVerification, 86400).await {
            Ok(raw_token) => {
                if let Err(e) = email_service
                    .send_email_verification(&user.email, &raw_token)
                    .await
                {
                    tracing::warn!(user_id = %user.id, error = %e, "failed to resend verification email");
                }
            }
            Err(e) => tracing::warn!(user_id = %user.id, error = %e.message, "failed to create verification token"),
        }
    }

    Ok(UserMutationResult::EmailVerifySuccess(EmailVerifySuccess {
        message: "Verification email sent".to_string(),
    }))
}
