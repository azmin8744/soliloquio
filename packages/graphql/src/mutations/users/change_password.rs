use super::{UserMutation, UserMutationResult, validation_errors_to_message};
use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::ChangePasswordInput;
use crate::utilities::requires_auth::RequiresAuth;
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_graphql::{Context, Result};
use models::{prelude::*, *};
use repositories::UserRepository;
use sea_orm::*;
use services::authentication::refresh_token::{cleanup_expired_tokens, revoke_all_refresh_tokens};
use services::validation::input_validator::InputValidator;

pub(super) async fn change_password(
    mutation: &UserMutation,
    ctx: &Context<'_>,
    input: ChangePasswordInput,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let current_user = match mutation.require_authenticate_as_user(ctx).await {
        Ok(user) => user,
        Err(e) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string(),
            }));
        }
    };

    if let Err(validation_errors) = input.validate() {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: validation_errors_to_message(validation_errors),
        }));
    }

    let parsed_hash = match PasswordHash::new(&current_user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Invalid password hash in database".to_string(),
            }));
        }
    };

    if Argon2::default()
        .verify_password(&input.current_password.into_bytes(), &parsed_hash)
        .is_err()
    {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: "Current password is incorrect".to_string(),
        }));
    }

    let temp_user = users::ActiveModel {
        id: ActiveValue::set(current_user.id),
        email: ActiveValue::set(current_user.email.clone()),
        password: ActiveValue::set(input.new_password.clone()),
        ..Default::default()
    };

    use services::validation::ActiveModelValidator;
    if let Err(validation_error) = temp_user.validate() {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: validation_error.to_string(),
        }));
    }

    if Argon2::default()
        .verify_password(&input.new_password.clone().into_bytes(), &parsed_hash)
        .is_ok()
    {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: "New password must be different from current password".to_string(),
        }));
    }

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let new_password_hash = match argon2.hash_password(&input.new_password.into_bytes(), &salt) {
        Ok(hash) => hash.to_string(),
        Err(_) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Failed to hash new password".to_string(),
            }));
        }
    };

    let user_id = current_user.id;
    if let Err(e) = UserRepository::update_password(db, user_id, new_password_hash).await {
        return Ok(UserMutationResult::DbError(DbError { message: e.to_string() }));
    }

    match revoke_all_refresh_tokens(db, user_id).await {
        Ok(_) => {}
        Err(_) => {
            tracing::warn!("failed to revoke refresh tokens on password change");
        }
    }

    let _ = cleanup_expired_tokens(db).await;

    tracing::info!(user_id = %user_id, "password changed");
    Ok(UserMutationResult::PasswordChangeSuccess(
        super::PasswordChangeSuccess {
            message: "Password changed successfully. Please sign in again on other devices."
                .to_string(),
        },
    ))
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

    #[tokio::test]
    async fn test_change_password_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = format!(
            r#"mutation {{ changePassword(input: {{ currentPassword: "{}", newPassword: "NewSecure@123!" }}) {{
                ... on AuthError {{ message }}
                ... on PasswordChangeSuccess {{ message }}
            }} }}"#,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["changePassword"]["message"]
            .as_str()
            .unwrap()
            .contains("Token not found"));
    }
}
