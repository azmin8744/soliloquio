use super::{UserMutation, UserMutationResult};
use crate::errors::{AuthError, DbError};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result, SimpleObject};
use sea_orm::DatabaseConnection;
use services::api_keys;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct CreateApiKeyResult {
    pub id: Uuid,
    pub label: String,
    pub raw_key: String,
}

pub(super) async fn create_api_key(
    mutation: &UserMutation,
    ctx: &Context<'_>,
    label: String,
) -> Result<UserMutationResult> {
    let user = match mutation.require_authenticate_as_user(ctx).await {
        Ok(u) => u,
        Err(e) => return Ok(UserMutationResult::AuthError(AuthError { message: e.to_string() })),
    };
    let db = ctx.data::<DatabaseConnection>().unwrap();
    let (raw_key, key_hash) = api_keys::generate();
    match api_keys::create(db, user.id, label.clone(), key_hash).await {
        Ok(record) => Ok(UserMutationResult::CreateApiKey(CreateApiKeyResult {
            id: record.id,
            label: record.label,
            raw_key,
        })),
        Err(e) => Ok(UserMutationResult::DbError(DbError { message: e.to_string() })),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use async_graphql::Request;
    use models::api_keys::Entity as ApiKeys;
    use sea_orm::EntityTrait;

    #[tokio::test]
    async fn test_create_api_key_success() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("cak_success");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = format!(
            r#"mutation {{ createApiKey(label: "my-key") {{
                ... on CreateApiKeyResult {{ id label rawKey }}
                ... on AuthError {{ message }}
                ... on DbError {{ message }}
            }} }}"#
        );

        let res = schema
            .execute(Request::new(&query).data(services::authentication::Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["createApiKey"]["id"].as_str().is_some());
        assert_eq!(data["createApiKey"]["label"], "my-key");
        assert!(data["createApiKey"]["rawKey"].as_str().is_some());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_create_api_key_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation { createApiKey(label: "key") {
            ... on AuthError { message }
            ... on CreateApiKeyResult { id }
        } }"#;

        let res = schema.execute(Request::new(query)).await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["createApiKey"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_create_api_key_raw_key_has_slq_prefix() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("cak_prefix");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation { createApiKey(label: "prefix-test") {
            ... on CreateApiKeyResult { rawKey }
        } }"#;

        let res = schema
            .execute(Request::new(query).data(services::authentication::Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();
        let raw_key = data["createApiKey"]["rawKey"].as_str().unwrap();
        assert!(raw_key.starts_with("slq_"), "Expected slq_ prefix, got: {}", raw_key);

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_create_api_key_stores_hash_not_raw_key() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("cak_hash");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation { createApiKey(label: "hash-test") {
            ... on CreateApiKeyResult { id rawKey }
        } }"#;

        let res = schema
            .execute(Request::new(query).data(services::authentication::Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();
        let raw_key = data["createApiKey"]["rawKey"].as_str().unwrap().to_string();
        let id: Uuid = data["createApiKey"]["id"].as_str().unwrap().parse().unwrap();

        let record = ApiKeys::find_by_id(id).one(&db).await.unwrap().unwrap();
        assert_ne!(record.key_hash, raw_key);

        cleanup_test_user(&db, user.id).await;
    }
}
