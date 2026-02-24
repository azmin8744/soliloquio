use crate::errors::AuthError;
use async_graphql::{Context, Result};
use sea_orm::DatabaseConnection;
use services::authentication::refresh_token::{cleanup_expired_tokens, revoke_all_refresh_tokens};
use services::authentication::token::Token;

pub(super) async fn logout_all_devices(
    ctx: &Context<'_>,
    access_token: String,
) -> Result<bool, AuthError> {
    let db = ctx.data::<DatabaseConnection>().unwrap();

    let token = Token::new(access_token);
    let user = services::authentication::authenticator::get_user(db, &token).await?;

    revoke_all_refresh_tokens(db, user.id)
        .await
        .map_err(|e| AuthError { message: e.message })?;

    let _ = cleanup_expired_tokens(db).await;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

    #[tokio::test]
    async fn test_logout_all_revokes_all_user_tokens() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("logout_all");
        let password = valid_password();

        create_test_user_with_password(&db, &email, &password).await;

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

        let logout_query = format!(
            r#"mutation {{ logoutAllDevices(accessToken: "{}") }}"#,
            access_token
        );

        let res = schema.execute(Request::new(&logout_query)).await;
        assert!(res.errors.is_empty());

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
}
