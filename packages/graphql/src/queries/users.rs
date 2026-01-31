use async_graphql::{Context, Object, Result};
use crate::types::user::User as UserType;
use crate::utilities::requires_auth::RequiresAuth;
use crate::errors::AuthError;

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
                created_at: user.created_at,
                updated_at: user.updated_at,
            })),
            Err(_) => Ok(None),
        }
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
        assert!(res.errors[0]
            .message
            .contains("Unknown field \"password\""));

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
}
