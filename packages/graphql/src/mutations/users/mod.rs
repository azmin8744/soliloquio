use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::{ChangePasswordInput, SignInInput, SignUpInput, UpdateUserInput};
use crate::types::authorized_user::AuthorizedUser;
use crate::types::user::User;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Object, Result, SimpleObject, Union};
use uuid::Uuid;

mod create_api_key;
mod revoke_api_key;
mod change_password;
mod forgot_password;
mod logout;
mod logout_all_devices;
mod refresh_access_token;
mod resend_verification_email;
mod reset_password;
mod sign_in;
mod sign_up;
mod update_user;
mod verify_email;

pub use create_api_key::CreateApiKeyResult;
pub use revoke_api_key::RevokeApiKeyResult;

#[derive(SimpleObject)]
pub struct PasswordChangeSuccess {
    pub message: String,
}

#[derive(SimpleObject)]
pub struct PasswordResetSuccess {
    pub message: String,
}

#[derive(SimpleObject)]
pub struct EmailVerifySuccess {
    pub message: String,
}

#[derive(Union)]
pub enum UserMutationResult {
    AuthorizedUser(AuthorizedUser),
    ValidationError(ValidationErrorType),
    DbError(DbError),
    AuthError(AuthError),
    PasswordChangeSuccess(PasswordChangeSuccess),
    PasswordResetSuccess(PasswordResetSuccess),
    EmailVerifySuccess(EmailVerifySuccess),
    User(User),
    CreateApiKey(CreateApiKeyResult),
    RevokeApiKey(RevokeApiKeyResult),
}

pub(super) fn validation_errors_to_message(
    errors: services::validation::input_validator::ValidationErrors,
) -> String {
    errors
        .values()
        .flatten()
        .cloned()
        .collect::<Vec<String>>()
        .join(", ")
}

#[derive(Default)]
pub struct UserMutation;

impl RequiresAuth for UserMutation {}

#[Object]
impl UserMutation {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<UserMutationResult> {
        sign_up::sign_up(ctx, input).await
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<UserMutationResult> {
        sign_in::sign_in(ctx, input).await
    }

    async fn refresh_access_token(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> Result<UserMutationResult> {
        refresh_access_token::refresh_access_token(ctx, refresh_token).await
    }

    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError> {
        logout::logout(ctx, refresh_token).await
    }

    async fn logout_all_devices(
        &self,
        ctx: &Context<'_>,
        access_token: String,
    ) -> Result<bool, AuthError> {
        logout_all_devices::logout_all_devices(ctx, access_token).await
    }

    async fn change_password(
        &self,
        ctx: &Context<'_>,
        input: ChangePasswordInput,
    ) -> Result<UserMutationResult> {
        change_password::change_password(self, ctx, input).await
    }

    async fn update_user(
        &self,
        ctx: &Context<'_>,
        input: UpdateUserInput,
    ) -> Result<UserMutationResult> {
        update_user::update_user(self, ctx, input).await
    }

    async fn forgot_password(
        &self,
        ctx: &Context<'_>,
        email: String,
    ) -> Result<UserMutationResult> {
        forgot_password::forgot_password(ctx, email).await
    }

    async fn reset_password(
        &self,
        ctx: &Context<'_>,
        token: String,
        new_password: String,
    ) -> Result<UserMutationResult> {
        reset_password::reset_password(ctx, token, new_password).await
    }

    async fn verify_email(&self, ctx: &Context<'_>, token: String) -> Result<UserMutationResult> {
        verify_email::verify_email(ctx, token).await
    }

    async fn resend_verification_email(&self, ctx: &Context<'_>) -> Result<UserMutationResult> {
        resend_verification_email::resend_verification_email(self, ctx).await
    }

    async fn create_api_key(&self, ctx: &Context<'_>, label: String) -> Result<UserMutationResult> {
        create_api_key::create_api_key(self, ctx, label).await
    }

    async fn revoke_api_key(&self, ctx: &Context<'_>, id: Uuid) -> Result<UserMutationResult> {
        revoke_api_key::revoke_api_key(self, ctx, id).await
    }
}
