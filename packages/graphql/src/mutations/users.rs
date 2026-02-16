use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::{ChangePasswordInput, SignInInput, SignUpInput};
use crate::types::authorized_user::AuthorizedUser;
use crate::utilities::requires_auth::RequiresAuth;
use actix_web::cookie::{Cookie, SameSite};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use async_graphql::{Context, Object, Result, SimpleObject, Union};
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;
use services::authentication::refresh_token::{
    cleanup_expired_tokens, create_refresh_token, revoke_all_refresh_tokens, revoke_refresh_token,
    validate_refresh_token,
};
use services::authentication::token::{generate_token, Token};
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
fn validation_errors_to_message(
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

trait UserMutations {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<UserMutationResult>;
    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<UserMutationResult>;
    async fn refresh_access_token(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> Result<UserMutationResult>;
    async fn change_password(
        &self,
        ctx: &Context<'_>,
        input: ChangePasswordInput,
    ) -> Result<UserMutationResult>;
    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError>;
    async fn logout_all_devices(
        &self,
        ctx: &Context<'_>,
        access_token: String,
    ) -> Result<bool, AuthError>;
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
            Err(_) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: "Failed to hash password".to_string(),
                }))
            }
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
            Err(e) => {
                tracing::warn!("signup DB error");
                return Ok(UserMutationResult::DbError(DbError {
                    message: e.to_string()
                }));
            }
        };

        // Create refresh token
        let refresh_token = match create_refresh_token(db, res.id, None).await {
            Ok(token) => token,
            Err(e) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }))
            }
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

        tracing::info!(user_id = %res.id, "signup success");
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
            Ok(None) => {
                tracing::warn!("signin failed: email not found");
                return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                    message: "Email or password incorrect".to_string()
                }));
            }
            Err(e) => return Ok(UserMutationResult::DbError(DbError {
                message: e.to_string()
            })),
        };

        // Verify password
        let parsed_hash = match PasswordHash::new(&user.password) {
            Ok(hash) => hash,
            Err(_) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: "Invalid password hash in database".to_string(),
                }))
            }
        };

        if Argon2::default().verify_password(&input.password.into_bytes(), &parsed_hash).is_err() {
            tracing::warn!("signin failed: wrong password");
            return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                message: "Email or password incorrect".to_string()
            }));
        }

        // Create refresh token and return success
        let refresh_token = match create_refresh_token(db, user.id, None).await {
            Ok(token) => token,
            Err(e) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }))
            }
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
        
        tracing::info!(user_id = %user.id, "signin success");
        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: access_token,
            refresh_token,
        }))
    }

    async fn refresh_access_token(
        &self,
        ctx: &Context<'_>,
        refresh_token: String,
    ) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Validate the refresh token and get the refresh token record
        let refresh_token_record = match validate_refresh_token(db, &refresh_token).await {
            Ok(record) => record,
            Err(e) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: e.message,
                }))
            }
        };

        // Get the user associated with this refresh token
        let user = match services::authentication::authenticator::get_user(
            db,
            &Token::new(refresh_token.clone()),
        )
        .await
        {
            Ok(user) => user,
            Err(e) => {
                return Ok(UserMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }))
            }
        };

        // Ensure the token belongs to the correct user (extra security check)
        if refresh_token_record.user_id != user.id {
            tracing::warn!("refresh token user_id mismatch");
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Refresh token does not belong to the authenticated user".to_string(),
            }));
        }

        // Generate a new access token
        let new_access_token = generate_token(&user);

        // Set httpOnly cookie for new access token
        let access_cookie = Cookie::build("access_token", &new_access_token)
            .http_only(true)
            .secure(false) // Set to true in production with HTTPS
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(actix_web::cookie::time::Duration::hours(1))
            .finish();

        ctx.insert_http_header("Set-Cookie", access_cookie.to_string());

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
        revoke_refresh_token(db, &refresh_token)
            .await
            .map_err(|e| AuthError { message: e.message })?;

        // Clear cookies by setting expired ones
        let expired_access = Cookie::build("access_token", "")
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(0))
            .finish();

        let expired_refresh = Cookie::build("refresh_token", "")
            .http_only(true)
            .secure(false)
            .same_site(SameSite::Lax)
            .path("/")
            .max_age(actix_web::cookie::time::Duration::seconds(0))
            .finish();

        ctx.insert_http_header("Set-Cookie", expired_access.to_string());
        ctx.append_http_header("Set-Cookie", expired_refresh.to_string());

        tracing::info!("logout");

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(true)
    }

    async fn logout_all_devices(
        &self,
        ctx: &Context<'_>,
        access_token: String,
    ) -> Result<bool, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Get user from access token
        let token = Token::new(access_token);
        let user = services::authentication::authenticator::get_user(db, &token).await?;

        // Revoke all refresh tokens for this user
        revoke_all_refresh_tokens(db, user.id)
            .await
            .map_err(|e| AuthError { message: e.message })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(true)
    }

    async fn change_password(
        &self,
        ctx: &Context<'_>,
        input: ChangePasswordInput,
    ) -> Result<UserMutationResult> {
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
        let new_password_hash = match argon2.hash_password(&input.new_password.into_bytes(), &salt)
        {
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
            Ok(_) => {}
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
            Err(_e) => {
                tracing::warn!("failed to revoke refresh tokens on password change");
            }
        }

        // 9. Cleanup expired tokens
        let _ = cleanup_expired_tokens(db).await;

        tracing::info!(user_id = %user_id, "password changed");
        Ok(UserMutationResult::PasswordChangeSuccess(PasswordChangeSuccess {
            message: "Password changed successfully. Please sign in again on other devices.".to_string(),
        }))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use async_graphql::Request;
    use models::refresh_tokens::{self, Entity as RefreshTokens};

    // ============= sign_up tests =============

    #[tokio::test]
    async fn test_signup_valid_input_creates_user() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signup_valid");

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
                ... on ValidationErrorType {{ message }}
                ... on DbError {{ message }}
            }} }}"#,
            email,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["signUp"]["token"].as_str().is_some());
        assert!(data["signUp"]["refreshToken"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_signup_returns_access_and_refresh_tokens() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signup_tokens");

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
            }} }}"#,
            email,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        let token = data["signUp"]["token"].as_str().unwrap();
        let refresh_token = data["signUp"]["refreshToken"].as_str().unwrap();

        assert!(!token.is_empty());
        assert!(!refresh_token.is_empty());
        assert_ne!(token, refresh_token);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_signup_hashes_password_with_argon2() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signup_hash");
        let password = valid_password();

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email, password
        );

        schema.execute(Request::new(&query)).await;

        // Check user in DB has argon2 hash
        let user = Users::find()
            .filter(users::Column::Email.eq(&email))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        assert!(user.password.starts_with("$argon2"));
        assert_ne!(user.password, password);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_signup_invalid_email_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "not-an-email", password: "{}" }}) {{
                ... on ValidationErrorType {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["signUp"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_signup_weak_password_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signup_weak");

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "weak" }}) {{
                ... on ValidationErrorType {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["signUp"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_signup_duplicate_email_returns_db_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signup_dup");

        // First signup
        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email,
            valid_password()
        );
        schema.execute(Request::new(&query)).await;

        // Second signup with same email
        let query2 = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on DbError {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email,
            valid_password()
        );

        let res = schema.execute(Request::new(&query2)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["signUp"]["message"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_signup_creates_refresh_token_in_db() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signup_rt");

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email,
            valid_password()
        );

        schema.execute(Request::new(&query)).await;

        let user = Users::find()
            .filter(users::Column::Email.eq(&email))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        let token_count = RefreshTokens::find()
            .filter(refresh_tokens::Column::UserId.eq(user.id))
            .count(&db)
            .await
            .unwrap();

        assert!(token_count >= 1);

        cleanup_test_user_by_email(&db, &email).await;
    }

    // ============= sign_in tests =============

    #[tokio::test]
    async fn test_signin_valid_credentials_returns_tokens() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signin_valid");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        let query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
                ... on ValidationErrorType {{ message }}
            }} }}"#,
            email, password
        );

        let res = schema.execute(Request::new(&query)).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["signIn"]["token"].as_str().is_some());
        assert!(data["signIn"]["refreshToken"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_signin_wrong_password_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signin_wrong_pw");

        create_test_user_with_password(&db, &email, &valid_password()).await;

        let query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "WrongP@ss123!" }}) {{
                ... on ValidationErrorType {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        let msg = data["signIn"]["message"].as_str().unwrap();
        assert_eq!(msg, "Email or password incorrect");

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_signin_nonexistent_email_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = format!(
            r#"mutation {{ signIn(input: {{ email: "nonexistent_{}@example.com", password: "{}" }}) {{
                ... on ValidationErrorType {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            uuid::Uuid::new_v4(),
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        // Should return same message to prevent email enumeration
        let msg = data["signIn"]["message"].as_str().unwrap();
        assert_eq!(msg, "Email or password incorrect");
    }

    #[tokio::test]
    async fn test_signin_empty_email_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = format!(
            r#"mutation {{ signIn(input: {{ email: "", password: "{}" }}) {{
                ... on ValidationErrorType {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["signIn"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_signin_empty_password_returns_validation_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation { signIn(input: { email: "test@example.com", password: "" }) {
            ... on ValidationErrorType { message }
            ... on AuthorizedUser { token }
        } }"#;

        let res = schema.execute(Request::new(query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["signIn"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_signin_creates_new_refresh_token() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("signin_new_rt");
        let password = valid_password();

        let user = create_test_user_with_password(&db, &email, &password).await;

        let before_count = RefreshTokens::find()
            .filter(refresh_tokens::Column::UserId.eq(user.id))
            .count(&db)
            .await
            .unwrap();

        let query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email, password
        );

        schema.execute(Request::new(&query)).await;

        let after_count = RefreshTokens::find()
            .filter(refresh_tokens::Column::UserId.eq(user.id))
            .count(&db)
            .await
            .unwrap();

        assert_eq!(after_count, before_count + 1);

        cleanup_test_user_by_email(&db, &email).await;
    }

    // ============= refresh_access_token tests =============

    #[tokio::test]
    async fn test_refresh_valid_returns_new_access_token() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("refresh_valid");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        // Sign in to get tokens
        let signin_query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
            }} }}"#,
            email, password
        );

        let signin_res = schema.execute(Request::new(&signin_query)).await;
        let signin_data = signin_res.data.into_json().unwrap();
        let refresh_token = signin_data["signIn"]["refreshToken"].as_str().unwrap();

        // Refresh
        let refresh_query = format!(
            r#"mutation {{ refreshAccessToken(refreshToken: "{}") {{
                ... on AuthorizedUser {{ token refreshToken }}
                ... on AuthError {{ message }}
            }} }}"#,
            refresh_token
        );

        let res = schema.execute(Request::new(&refresh_query)).await;
        let data = res.data.into_json().unwrap();

        let new_token = data["refreshAccessToken"]["token"].as_str().unwrap();
        assert!(!new_token.is_empty());
        // New token might be different (different iat/jti)
        assert!(new_token.len() > 0);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_refresh_returns_same_refresh_token() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("refresh_same_rt");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        let signin_query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ refreshToken }}
            }} }}"#,
            email, password
        );

        let signin_res = schema.execute(Request::new(&signin_query)).await;
        let signin_data = signin_res.data.into_json().unwrap();
        let refresh_token = signin_data["signIn"]["refreshToken"].as_str().unwrap();

        let refresh_query = format!(
            r#"mutation {{ refreshAccessToken(refreshToken: "{}") {{
                ... on AuthorizedUser {{ refreshToken }}
            }} }}"#,
            refresh_token
        );

        let res = schema.execute(Request::new(&refresh_query)).await;
        let data = res.data.into_json().unwrap();

        let returned_rt = data["refreshAccessToken"]["refreshToken"].as_str().unwrap();
        assert_eq!(returned_rt, refresh_token);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_refresh_invalid_token_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation { refreshAccessToken(refreshToken: "invalid.token.here") {
            ... on AuthError { message }
            ... on AuthorizedUser { token }
        } }"#;

        let res = schema.execute(Request::new(query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["refreshAccessToken"]["message"].as_str().is_some());
    }

    // ============= logout tests =============

    #[tokio::test]
    async fn test_logout_revokes_specific_token() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("logout_revoke");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        let signin_query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ refreshToken }}
            }} }}"#,
            email, password
        );

        let signin_res = schema.execute(Request::new(&signin_query)).await;
        let signin_data = signin_res.data.into_json().unwrap();
        let refresh_token = signin_data["signIn"]["refreshToken"].as_str().unwrap();

        let logout_query = format!(
            r#"mutation {{ logout(refreshToken: "{}") }}"#,
            refresh_token
        );

        let res = schema.execute(Request::new(&logout_query)).await;
        assert!(res.errors.is_empty());

        let data = res.data.into_json().unwrap();
        assert_eq!(data["logout"], true);

        // Token should now be invalid
        let refresh_query = format!(
            r#"mutation {{ refreshAccessToken(refreshToken: "{}") {{
                ... on AuthError {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            refresh_token
        );

        let refresh_res = schema.execute(Request::new(&refresh_query)).await;
        let refresh_data = refresh_res.data.into_json().unwrap();
        assert!(refresh_data["refreshAccessToken"]["message"]
            .as_str()
            .is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_logout_leaves_other_sessions_active() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("logout_other");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        // Sign in twice to create 2 refresh tokens
        let signin_query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ refreshToken }}
            }} }}"#,
            email, password
        );

        let res1 = schema.execute(Request::new(&signin_query)).await;
        let data1 = res1.data.into_json().unwrap();
        let token1 = data1["signIn"]["refreshToken"].as_str().unwrap();

        let res2 = schema.execute(Request::new(&signin_query)).await;
        let data2 = res2.data.into_json().unwrap();
        let token2 = data2["signIn"]["refreshToken"].as_str().unwrap();

        // Logout token1
        let logout_query = format!(r#"mutation {{ logout(refreshToken: "{}") }}"#, token1);
        schema.execute(Request::new(&logout_query)).await;

        // token2 should still work
        let refresh_query = format!(
            r#"mutation {{ refreshAccessToken(refreshToken: "{}") {{
                ... on AuthorizedUser {{ token }}
                ... on AuthError {{ message }}
            }} }}"#,
            token2
        );

        let res = schema.execute(Request::new(&refresh_query)).await;
        let data = res.data.into_json().unwrap();
        assert!(data["refreshAccessToken"]["token"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_logout_nonexistent_token_returns_true() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation { logout(refreshToken: "nonexistent.fake.token") }"#;

        let res = schema.execute(Request::new(query)).await;
        assert!(res.errors.is_empty());

        let data = res.data.into_json().unwrap();
        assert_eq!(data["logout"], true);
    }

    // ============= logout_all_devices tests =============

    #[tokio::test]
    async fn test_logout_all_revokes_all_user_tokens() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("logout_all");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        // Sign in twice
        let signin_query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
            }} }}"#,
            email, password
        );

        let res1 = schema.execute(Request::new(&signin_query)).await;
        let data1 = res1.data.into_json().unwrap();
        let access_token = data1["signIn"]["token"].as_str().unwrap();
        let refresh1 = data1["signIn"]["refreshToken"].as_str().unwrap();

        schema.execute(Request::new(&signin_query)).await;

        // Logout all
        let logout_query = format!(
            r#"mutation {{ logoutAllDevices(accessToken: "{}") }}"#,
            access_token
        );

        let res = schema.execute(Request::new(&logout_query)).await;
        assert!(res.errors.is_empty());

        // Both refresh tokens should be invalid
        let refresh_query = format!(
            r#"mutation {{ refreshAccessToken(refreshToken: "{}") {{
                ... on AuthError {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            refresh1
        );

        let res = schema.execute(Request::new(&refresh_query)).await;
        let data = res.data.into_json().unwrap();
        assert!(data["refreshAccessToken"]["message"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_logout_all_invalid_access_token_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation { logoutAllDevices(accessToken: "invalid.access.token") }"#;

        let res = schema.execute(Request::new(query)).await;
        assert!(!res.errors.is_empty());
    }

    // ============= change_password tests =============

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
