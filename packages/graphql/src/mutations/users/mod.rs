use async_graphql::MergedObject;

mod change_password;
mod create_api_key;
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
mod revoke_api_key;

pub use change_password::PasswordChangeSuccess;
pub use verify_email::EmailVerifySuccess;

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

#[derive(MergedObject, Default)]
pub struct UserMutation(
    sign_up::SignUpMutation,
    sign_in::SignInMutation,
    refresh_access_token::RefreshAccessTokenMutation,
    logout::LogoutMutation,
    logout_all_devices::LogoutAllDevicesMutation,
    change_password::ChangePasswordMutation,
    forgot_password::ForgotPasswordMutation,
    reset_password::ResetPasswordMutation,
    verify_email::VerifyEmailMutation,
    resend_verification_email::ResendVerificationEmailMutation,
    update_user::UpdateUserMutation,
    create_api_key::CreateApiKeyMutation,
    revoke_api_key::RevokeApiKeyMutation,
);
