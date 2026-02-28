use super::{UserMutation, UserMutationResult};
use crate::errors::{AuthError, DbError};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result, SimpleObject};
use sea_orm::DatabaseConnection;
use services::api_keys;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct RevokeApiKeyResult {
    pub id: Uuid,
}

pub(super) async fn revoke_api_key(
    mutation: &UserMutation,
    ctx: &Context<'_>,
    id: Uuid,
) -> Result<UserMutationResult> {
    let user = match mutation.require_authenticate_as_user(ctx).await {
        Ok(u) => u,
        Err(e) => return Ok(UserMutationResult::AuthError(AuthError { message: e.to_string() })),
    };
    let db = ctx.data::<DatabaseConnection>().unwrap();
    match api_keys::revoke(db, id, user.id).await {
        Ok(_) => Ok(UserMutationResult::RevokeApiKey(RevokeApiKeyResult { id })),
        Err(e) => Ok(UserMutationResult::DbError(DbError { message: e.to_string() })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use async_graphql::Request;

    #[tokio::test]
    async fn test_revoke_api_key_success() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("rak_success");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let (raw_key, key_hash) = services::api_keys::generate();
        let record = services::api_keys::create(&db, user.id, "test-key".to_string(), key_hash)
            .await
            .unwrap();
        let _ = raw_key;

        let query = format!(
            r#"mutation {{ revokeApiKey(id: "{}") {{
                ... on RevokeApiKeyResult {{ id }}
                ... on AuthError {{ message }}
                ... on DbError {{ message }}
            }} }}"#,
            record.id
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert_eq!(data["revokeApiKey"]["id"].as_str().unwrap(), record.id.to_string());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_revoke_api_key_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = format!(
            r#"mutation {{ revokeApiKey(id: "{}") {{
                ... on AuthError {{ message }}
                ... on RevokeApiKeyResult {{ id }}
            }} }}"#,
            Uuid::new_v4()
        );

        let res = schema.execute(Request::new(&query)).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["revokeApiKey"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_revoke_api_key_wrong_user_returns_db_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let email1 = generate_unique_email("rak_owner");
        let user1 = create_test_user_with_password(&db, &email1, &valid_password()).await;

        let email2 = generate_unique_email("rak_attacker");
        let user2 = create_test_user_with_password(&db, &email2, &valid_password()).await;
        let token2 = create_access_token(&user2);

        let (raw_key, key_hash) = services::api_keys::generate();
        let record = services::api_keys::create(&db, user1.id, "owner-key".to_string(), key_hash)
            .await
            .unwrap();
        let _ = raw_key;

        let query = format!(
            r#"mutation {{ revokeApiKey(id: "{}") {{
                ... on DbError {{ message }}
                ... on RevokeApiKeyResult {{ id }}
            }} }}"#,
            record.id
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token2)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["revokeApiKey"]["message"].as_str().is_some());

        cleanup_test_user(&db, user1.id).await;
        cleanup_test_user(&db, user2.id).await;
    }
}
