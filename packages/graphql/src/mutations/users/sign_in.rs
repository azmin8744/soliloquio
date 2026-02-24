use super::{UserMutationResult, validation_errors_to_message};
use crate::errors::{AuthError, DbError, ValidationErrorType};
use crate::mutations::input_validators::SignInInput;
use crate::types::authorized_user::AuthorizedUser;
use actix_web::cookie::{Cookie, SameSite};
use argon2::{
    password_hash::{PasswordHash, PasswordVerifier},
    Argon2,
};
use async_graphql::{Context, Result};
use models::{prelude::*, *};
use repositories::UserRepository;
use sea_orm::*;
use services::authentication::refresh_token::{cleanup_expired_tokens, create_refresh_token};
use services::authentication::token::generate_token;
use services::validation::input_validator::InputValidator;

pub(super) async fn sign_in(
    ctx: &Context<'_>,
    input: SignInInput,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    if let Err(validation_errors) = input.validate() {
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: validation_errors_to_message(validation_errors),
        }));
    }

    let user = match UserRepository::find_by_email_for_login(db, &input.email).await {
        Ok(Some(user)) => user,
        Ok(None) => {
            tracing::warn!("signin failed: email not found");
            return Ok(UserMutationResult::ValidationError(ValidationErrorType {
                message: "Email or password incorrect".to_string(),
            }));
        }
        Err(e) => return Ok(UserMutationResult::DbError(DbError { message: e.to_string() })),
    };

    let parsed_hash = match PasswordHash::new(&user.password) {
        Ok(hash) => hash,
        Err(_) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Invalid password hash in database".to_string(),
            }))
        }
    };

    if Argon2::default()
        .verify_password(&input.password.into_bytes(), &parsed_hash)
        .is_err()
    {
        tracing::warn!("signin failed: wrong password");
        return Ok(UserMutationResult::ValidationError(ValidationErrorType {
            message: "Email or password incorrect".to_string(),
        }));
    }

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

    tracing::info!(user_id = %user.id, "signin success");
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
}
