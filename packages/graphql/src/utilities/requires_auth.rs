use std::fmt;
use services::authentication::token::Token;
use async_graphql::{Context, Result};

#[derive(Debug)]
pub struct AuthenticationError {
    pub message: String,
}

impl From<services::authentication::authenticator::AuthenticationError> for AuthenticationError {
    fn from(e: services::authentication::authenticator::AuthenticationError) -> Self {
        AuthenticationError { message: e.to_string() }
    }
}
impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

pub trait RequiresAuth {
    async fn require_authenticate_as_user<'a>(&self, ctx: &Context<'a>) -> Result<models::users::Model, AuthenticationError> {
        let token = match ctx.data::<Token>() {
            Ok(token) => token,
            Err(_) => {
                return Err(AuthenticationError {
                    message: "Token not found".to_string(),
                });
            }
        };

        let db = ctx.data::<sea_orm::DatabaseConnection>().unwrap();
        let user = services::authentication::authenticator::get_user(db, token).await?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;
    use services::authentication::Token;

    // Tests for require_authenticate_as_user through the posts query
    // (which uses RequiresAuth trait)

    #[tokio::test]
    async fn test_require_auth_valid_token_returns_user() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("auth_valid");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        // posts query requires auth - if it works, auth succeeded
        let query = r#"query { posts { id } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_require_auth_missing_token_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"query { posts { id } }"#;

        let res = schema.execute(Request::new(query)).await;
        assert!(!res.errors.is_empty());
        // Error should indicate token not found
        assert!(res.errors[0].message.contains("Token not found"));
    }

    #[tokio::test]
    async fn test_require_auth_invalid_token_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"query { posts { id } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new("invalid.jwt.token".to_string())))
            .await;
        assert!(!res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_require_auth_expired_token_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("auth_expired");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;

        // Create an expired token
        let expired_token = create_expired_access_token(&user);

        let query = r#"query { posts { id } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(expired_token)))
            .await;
        assert!(!res.errors.is_empty());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_require_auth_deleted_user_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("auth_deleted");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        // Delete the user
        cleanup_test_user(&db, user.id).await;

        let query = r#"query { posts { id } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(!res.errors.is_empty());
        assert!(res.errors[0].message.contains("User not found"));
    }
}
