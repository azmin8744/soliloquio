use super::UserMutationResult;
use crate::errors::AuthError;
use crate::types::authorized_user::AuthorizedUser;
use actix_web::cookie::{Cookie, SameSite};
use async_graphql::{Context, Result};
use sea_orm::DatabaseConnection;
use services::authentication::refresh_token::{cleanup_expired_tokens, validate_refresh_token};
use services::authentication::token::{generate_token, Token};

pub(super) async fn refresh_access_token(
    ctx: &Context<'_>,
    refresh_token: String,
) -> Result<UserMutationResult> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let refresh_token_record = match validate_refresh_token(db, &refresh_token).await {
        Ok(record) => record,
        Err(e) => {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: e.message,
            }))
        }
    };

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

    if refresh_token_record.user_id != user.id {
        tracing::warn!("refresh token user_id mismatch");
        return Ok(UserMutationResult::AuthError(AuthError {
            message: "Refresh token does not belong to the authenticated user".to_string(),
        }));
    }

    let new_access_token = generate_token(&user);

    let access_cookie = Cookie::build("access_token", &new_access_token)
        .http_only(true)
        .secure(false)
        .same_site(SameSite::Lax)
        .path("/")
        .max_age(actix_web::cookie::time::Duration::hours(1))
        .finish();

    ctx.insert_http_header("Set-Cookie", access_cookie.to_string());

    let _ = cleanup_expired_tokens(db).await;

    Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
        token: new_access_token,
        refresh_token,
    }))
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

    #[tokio::test]
    async fn test_refresh_valid_returns_new_access_token() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("refresh_valid");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

        let signin_query = format!(
            r#"mutation {{ signIn(input: {{ email: "{}", password: "{}" }}) {{
                ... on AuthorizedUser {{ token refreshToken }}
            }} }}"#,
            email, password
        );

        let signin_res = schema.execute(Request::new(&signin_query)).await;
        let signin_data = signin_res.data.into_json().unwrap();
        let refresh_token = signin_data["signIn"]["refreshToken"].as_str().unwrap();

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
}
