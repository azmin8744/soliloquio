use crate::errors::AuthError;
use crate::types::api_key::ApiKeyInfo;
use crate::types::user::User as UserType;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Object, Result};
use sea_orm::DatabaseConnection;
use services::api_keys;

#[derive(Default)]
pub struct UserQueries;

impl RequiresAuth for UserQueries {}

#[Object]
impl UserQueries {
    /// Get the currently authenticated user's profile
    async fn me(&self, ctx: &Context<'_>) -> Result<Option<UserType>, AuthError> {
        match self.require_authenticate_as_user(ctx).await {
            Ok(user) => Ok(Some(UserType {
                id: user.id,
                email: user.email,
                email_verified_at: user.email_verified_at,
                display_name: user.display_name,
                bio: user.bio,
                created_at: user.created_at,
                updated_at: user.updated_at,
            })),
            Err(_) => Ok(None),
        }
    }

    async fn api_keys(&self, ctx: &Context<'_>) -> Result<Vec<ApiKeyInfo>, AuthError> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let keys = api_keys::list(db, user.id)
            .await
            .map_err(|e| AuthError { message: e.to_string() })?;
        Ok(keys
            .into_iter()
            .map(|k| ApiKeyInfo {
                id: k.id,
                label: k.label,
                last_used_at: k.last_used_at,
                created_at: k.created_at,
            })
            .collect())
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;
    use services::authentication::Token;

    #[tokio::test]
    async fn test_me_authenticated_returns_user() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("me_auth");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"query { me { id email } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert_eq!(data["me"]["email"], email);
        assert_eq!(data["me"]["id"], user.id.to_string());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_me_unauthenticated_returns_none() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"query { me { id } }"#;

        let res = schema.execute(Request::new(query)).await;
        assert!(res.errors.is_empty());

        let data = res.data.into_json().unwrap();
        assert!(data["me"].is_null());
    }

    #[tokio::test]
    async fn test_me_excludes_password() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("me_no_pw");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        // Try to query password field - should fail at schema level
        let query = r#"query { me { id email password } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;

        // Should have error because password field doesn't exist
        assert!(!res.errors.is_empty());
        assert!(res.errors[0].message.contains("Unknown field \"password\""));

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_me_returns_timestamps() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("me_timestamps");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"query { me { id createdAt updatedAt } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty());

        let data = res.data.into_json().unwrap();
        assert!(data["me"]["createdAt"].as_str().is_some());
        // updatedAt can be null for new users
        // just verify the field exists in response
        assert!(data["me"].get("updatedAt").is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_api_keys_unauthenticated_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"query { apiKeys { id label } }"#;
        let res = schema.execute(Request::new(query)).await;

        assert!(!res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_api_keys_authenticated_no_keys_returns_empty() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("api_keys_empty");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"query { apiKeys { id label } }"#;
        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert_eq!(data["apiKeys"].as_array().unwrap().len(), 0);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_api_keys_authenticated_returns_keys() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("api_keys_list");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let (_, hash1) = services::api_keys::generate();
        services::api_keys::create(&db, user.id, "key-one".into(), hash1)
            .await
            .unwrap();
        let (_, hash2) = services::api_keys::generate();
        services::api_keys::create(&db, user.id, "key-two".into(), hash2)
            .await
            .unwrap();

        let query = r#"query { apiKeys { id label createdAt } }"#;
        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        let keys = data["apiKeys"].as_array().unwrap();
        assert_eq!(keys.len(), 2);

        let labels: Vec<&str> = keys.iter().map(|k| k["label"].as_str().unwrap()).collect();
        assert!(labels.contains(&"key-one"));
        assert!(labels.contains(&"key-two"));
        assert!(keys[0]["createdAt"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_api_keys_only_returns_own_keys() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email1 = generate_unique_email("api_keys_own1");
        let email2 = generate_unique_email("api_keys_own2");
        let user1 = create_test_user_with_password(&db, &email1, &valid_password()).await;
        let user2 = create_test_user_with_password(&db, &email2, &valid_password()).await;
        let token1 = create_access_token(&user1);
        let token2 = create_access_token(&user2);

        let (_, hash1) = services::api_keys::generate();
        services::api_keys::create(&db, user1.id, "user1-key".into(), hash1)
            .await
            .unwrap();
        let (_, hash2) = services::api_keys::generate();
        services::api_keys::create(&db, user2.id, "user2-key".into(), hash2)
            .await
            .unwrap();

        let query = r#"query { apiKeys { id label } }"#;

        let res1 = schema
            .execute(Request::new(query).data(Token::new(token1)))
            .await;
        assert!(res1.errors.is_empty(), "Errors: {:?}", res1.errors);
        let data1 = res1.data.into_json().unwrap();
        let keys1 = data1["apiKeys"].as_array().unwrap();
        assert_eq!(keys1.len(), 1);
        assert_eq!(keys1[0]["label"], "user1-key");

        let res2 = schema
            .execute(Request::new(query).data(Token::new(token2)))
            .await;
        assert!(res2.errors.is_empty(), "Errors: {:?}", res2.errors);
        let data2 = res2.data.into_json().unwrap();
        let keys2 = data2["apiKeys"].as_array().unwrap();
        assert_eq!(keys2.len(), 1);
        assert_eq!(keys2[0]["label"], "user2-key");

        cleanup_test_user_by_email(&db, &email1).await;
        cleanup_test_user_by_email(&db, &email2).await;
    }
}
