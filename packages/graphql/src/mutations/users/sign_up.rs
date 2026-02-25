use super::{UserMutationResult, validation_errors_to_message};
use crate::config::SingleUserMode;
use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::SignUpInput;
use crate::types::authorized_user::AuthorizedUser;
use actix_web::cookie::{Cookie, SameSite};
use argon2::{
    password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
    Argon2,
};
use async_graphql::{Context, Result};
use repositories::UserRepository;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;
use services::authentication::refresh_token::{cleanup_expired_tokens, create_refresh_token};
use services::authentication::token::generate_token;
use services::email::EmailService;
use services::validation::input_validator::InputValidator;
use services::verification_token::{create_token, TokenKind};

pub(super) async fn sign_up(
    ctx: &Context<'_>,
    input: SignUpInput,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    if let Err(validation_errors) = input.validate() {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: validation_errors_to_message(validation_errors),
        }));
    }

    let single_user_mode = ctx.data::<SingleUserMode>().map(|s| s.0).unwrap_or(false);
    if single_user_mode {
        let count = UserRepository::count(db).await.unwrap_or(0);
        if count >= 1 {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Registration is disabled".to_string(),
            }));
        }
    }

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

    let res = match UserRepository::create(db, Uuid::new_v4(), input.email, password_hash).await {
        Ok(user) => user,
        Err(e) => {
            tracing::warn!("signup DB error");
            return Ok(UserMutationResult::DbError(DbError {
                message: e.to_string(),
            }));
        }
    };

    let refresh_token = match create_refresh_token(db, res.id, None).await {
        Ok(token) => token,
        Err(e) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string(),
            }))
        }
    };

    let _ = cleanup_expired_tokens(db).await;

    if let Ok(email_service) = ctx.data::<EmailService>() {
        match create_token(db, res.id, TokenKind::EmailVerification, 86400).await {
            Ok(raw_token) => {
                if let Err(e) = email_service.send_email_verification(&res.email, &raw_token).await {
                    tracing::warn!(user_id = %res.id, error = %e, "failed to send verification email");
                }
            }
            Err(e) => tracing::warn!(user_id = %res.id, error = %e.message, "failed to create verification token"),
        }
    }

    let access_token = generate_token(&res);

    let access_cookie = Cookie::build("access_token", &access_token)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::hours(1))
        .finish();

    let refresh_cookie = Cookie::build("refresh_token", &refresh_token)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::days(7))
        .finish();

    ctx.insert_http_header("Set-Cookie", access_cookie.to_string());
    ctx.append_http_header("Set-Cookie", refresh_cookie.to_string());

    tracing::info!(user_id = %res.id, "signup success");
    Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
        token: access_token,
        refresh_token,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use async_graphql::Request;
    use models::refresh_tokens::{self, Entity as RefreshTokens};
    use models::prelude::Users;
    use models::users;

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

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            email,
            valid_password()
        );
        schema.execute(Request::new(&query)).await;

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

    #[tokio::test]
    async fn test_signup_single_user_mode_rejects_when_user_exists() {
        let db = setup_test_db().await;
        let schema = create_test_schema_single_user(db.clone());
        let existing_email = generate_unique_email("sum_existing");
        let existing_user =
            create_test_user_with_password(&db, &existing_email, &valid_password()).await;

        let new_email = generate_unique_email("sum_new");
        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token }}
                ... on AuthError {{ message }}
            }} }}"#,
            new_email,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        assert_eq!(data["signUp"]["message"], "Registration is disabled");

        cleanup_test_user(&db, existing_user.id).await;
    }

    #[tokio::test]
    async fn test_signup_single_user_mode_disabled_allows_signup() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("sum_disabled");

        let query = format!(
            r#"mutation {{ signUp(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
                ... on AuthError {{ message }}
            }} }}"#,
            email,
            valid_password()
        );

        let res = schema.execute(Request::new(&query)).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        assert!(data["signUp"]["token"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }
}
