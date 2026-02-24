use crate::errors::AuthError;
use actix_web::cookie::{Cookie, SameSite};
use async_graphql::{Context, Result};
use sea_orm::DatabaseConnection;
use services::authentication::refresh_token::{cleanup_expired_tokens, revoke_refresh_token};

pub(super) async fn logout(
    ctx: &Context<'_>,
    refresh_token: String,
) -> Result<bool, AuthError> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    revoke_refresh_token(db, &refresh_token)
        .await
        .map_err(|e| AuthError { message: e.message })?;

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

    let _ = cleanup_expired_tokens(db).await;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

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

        let refresh_query = format!(
            r#"mutation {{ refreshAccessToken(refreshToken: "{}") {{
                ... on AuthError {{ message }}
                ... on AuthorizedUser {{ token }}
            }} }}"#,
            refresh_token
        );

        let refresh_res = schema.execute(Request::new(&refresh_query)).await;
        let refresh_data = refresh_res.data.into_json().unwrap();
        assert!(refresh_data["refreshAccessToken"]["message"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_logout_leaves_other_sessions_active() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("logout_other");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

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

        let logout_query = format!(r#"mutation {{ logout(refreshToken: "{}") }}"#, token1);
        schema.execute(Request::new(&logout_query)).await;

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
}
