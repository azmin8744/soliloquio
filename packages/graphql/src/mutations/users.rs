use async_graphql::{Context, Object, Result, SimpleObject, Union};
use sea_orm::*;
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use actix_web::cookie::{Cookie, SameSite};
use crate::types::authorized_user::AuthorizedUser;
use crate::errors::{DbError, AuthError, ValidationErrorType};
use crate::utilities::requires_auth::RequiresAuth;
use crate::mutations::input_validators::{SignUpInput, SignInInput, ChangePasswordInput};
use services::authentication::token::{Token, generate_token};
use services::authentication::refresh_token::{
    create_refresh_token, validate_refresh_token, revoke_refresh_token, revoke_all_refresh_tokens, cleanup_expired_tokens
};
use services::validation::input_validator::InputValidator;

#[derive(SimpleObject)]
pub struct PasswordChangeSuccess {
    message: String,
}

#[derive(Union)]
pub enum UserMutationResult {
    AuthorizedUser(AuthorizedUser),
    ValidationError(ValidationErrorType),
    DbError(DbError),
    AuthError(AuthError),
    PasswordChangeSuccess(PasswordChangeSuccess),
}

// Helper function to convert validation errors to GraphQL response
fn validation_errors_to_message(errors: services::validation::input_validator::ValidationErrors) -> String {
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

trait UserMutations {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<UserMutationResult>;
    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<UserMutationResult>;
    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<UserMutationResult>;
    async fn change_password(&self, ctx: &Context<'_>, input: ChangePasswordInput) -> Result<UserMutationResult>;
    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError>;
    async fn logout_all_devices(&self, ctx: &Context<'_>, access_token: String) -> Result<bool, AuthError>;
}

#[Object]
impl UserMutations for UserMutation {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        
        // Validate input
        if let Err(validation_errors) = input.validate() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                message: validation_errors_to_message(validation_errors),
            }));
        }
        
        // Hash password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = match argon2.hash_password(&input.password.into_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(_) => return Ok(UserMutationResult::AuthError(AuthError {
                message: "Failed to hash password".to_string()
            })),
        };
        
        // Create user
        let user = users::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            email: ActiveValue::set(input.email),
            password: ActiveValue::set(password_hash),
            ..Default::default()
        };
        
        let res = match user.insert(db).await {
            Ok(user) => user,
            Err(e) => return Ok(UserMutationResult::DbError(DbError {
                message: e.to_string()
            })),
        };

        // Create refresh token
        let refresh_token = match create_refresh_token(db, res.id, None).await {
            Ok(token) => token,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string()
            })),
        };

        let _ = cleanup_expired_tokens(db).await;

        let access_token = generate_token(&res);
        
        // Set httpOnly cookies for both access and refresh tokens
        let access_cookie = Cookie::build("access_token", &access_token)
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(actix_web::cookie::time::Duration::hours(1))
            .finish();

        let refresh_cookie = Cookie::build("refresh_token", &refresh_token)
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(actix_web::cookie::time::Duration::days(7))
            .finish();

        // Set cookies via GraphQL context
        ctx.insert_http_header("Set-Cookie", access_cookie.to_string());
        ctx.append_http_header("Set-Cookie", refresh_cookie.to_string());

        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: access_token,
            refresh_token,
        }))
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        
        // Validate input
        if let Err(validation_errors) = input.validate() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                message: validation_errors_to_message(validation_errors),
            }));
        }
        
        // Find user
        let user = match Users::find()
            .filter(users::Column::Email.contains(&input.email))
            .one(db)
            .await 
        {
            Ok(Some(user)) => user,
            Ok(None) => return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: "Email or password incorrect".to_string() 
            })),
            Err(e) => return Ok(UserMutationResult::DbError(DbError {
                message: e.to_string()
            })),
        };

        // Verify password
        let parsed_hash = match PasswordHash::new(&user.password) {
            Ok(hash) => hash,
            Err(_) => return Ok(UserMutationResult::AuthError(AuthError { 
                message: "Invalid password hash in database".to_string() 
            })),
        };

        if Argon2::default().verify_password(&input.password.into_bytes(), &parsed_hash).is_err() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: "Email or password incorrect".to_string() 
            }));
        }

        // Create refresh token and return success
        let refresh_token = match create_refresh_token(db, user.id, None).await {
            Ok(token) => token,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string()
            })),
        };

        let _ = cleanup_expired_tokens(db).await;
        
        let access_token = generate_token(&user);
        
        // Set httpOnly cookies for both access and refresh tokens
        let access_cookie = Cookie::build("access_token", &access_token)
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax) // Lax instead of Strict for better compatibility
            .path("/")
            .max_age(actix_web::cookie::time::Duration::hours(1))
            .finish();

        let refresh_cookie = Cookie::build("refresh_token", &refresh_token)
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(actix_web::cookie::time::Duration::days(7))
            .finish();

        // Set cookies via GraphQL context
        ctx.insert_http_header("Set-Cookie", access_cookie.to_string());
        ctx.append_http_header("Set-Cookie", refresh_cookie.to_string());
        
        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: access_token,
            refresh_token,
        }))
    }

    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Validate the refresh token and get the refresh token record
        let refresh_token_record = match validate_refresh_token(db, &refresh_token).await {
            Ok(record) => record,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.message
            })),
        };

        // Get the user associated with this refresh token
        let user = match services::authentication::authenticator::get_user(db, &Token::new(refresh_token.clone())).await {
            Ok(user) => user,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string()
            })),
        };
        
        // Ensure the token belongs to the correct user (extra security check)
        if refresh_token_record.user_id != user.id {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Refresh token does not belong to the authenticated user".to_string(),
            }));
        }

        // Generate a new access token
        let new_access_token = generate_token(&user);

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: new_access_token,
            refresh_token: refresh_token, // Return the same refresh token
        }))
    }

    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Revoke the specific refresh token
        revoke_refresh_token(db, &refresh_token).await
            .map_err(|e| AuthError { message: e.message })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(true)
    }

    async fn logout_all_devices(&self, ctx: &Context<'_>, access_token: String) -> Result<bool, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Get user from access token
        let token = Token::new(access_token);
        let user = services::authentication::authenticator::get_user(db, &token).await?;

        // Revoke all refresh tokens for this user
        revoke_all_refresh_tokens(db, user.id).await
            .map_err(|e| AuthError { message: e.message })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(true)
    }

    async fn change_password(&self, ctx: &Context<'_>, input: ChangePasswordInput) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // 1. Authenticate user
        let current_user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        // 2. Validate input
        if let Err(validation_errors) = input.validate() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                message: validation_errors_to_message(validation_errors),
            }));
        }

        // 3. Verify current password
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

        // 4. Validate new password using existing validation
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

        // 5. Check if new password is different from current
        if Argon2::default()
            .verify_password(&input.new_password.clone().into_bytes(), &parsed_hash)
            .is_ok()
        {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                message: "New password must be different from current password".to_string(),
            }));
        }

        // 6. Hash new password
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

        // 7. Update password in database
        let user_id = current_user.id; // Store the ID before moving
        let mut user_to_update = current_user.into_active_model();
        user_to_update.password = ActiveValue::set(new_password_hash);
        user_to_update.updated_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));

        match Users::update(user_to_update).exec(db).await {
            Ok(_) => {},
            Err(e) => {
                return Ok(UserMutationResult::DbError(DbError {
                    message: e.to_string(),
                }));
            }
        }

        // 8. Optional: Revoke all refresh tokens except current session
        // This forces re-authentication on all other devices
        match revoke_all_refresh_tokens(db, user_id).await {
            Ok(_) => {},
            Err(e) => {
                // Log error but don't fail the password change
                eprintln!("Warning: Failed to revoke refresh tokens: {}", e.message);
            }
        }

        // 9. Cleanup expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(UserMutationResult::PasswordChangeSuccess(PasswordChangeSuccess {
            message: "Password changed successfully. Please sign in again on other devices.".to_string(),
        }))
    }
}
