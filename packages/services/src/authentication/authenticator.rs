use crate::authentication::token::Token;
use std::fmt;
use sea_orm::*;
use models::users::{Model, Entity as users};

#[derive(Debug)]
pub struct BadCredentialsError {
    pub message: String,
}

#[derive(Debug)]
pub struct DbError {
    pub message: String,
}

#[derive(Debug)]
pub enum AuthenticationError {
    BadCredentials(BadCredentialsError),
    DbError(DbError),
}

impl From<crate::authentication::AuthError> for AuthenticationError {
    fn from(e: crate::authentication::token::AuthError) -> Self {
        AuthenticationError::BadCredentials(BadCredentialsError {
            message: e.to_string(),
        })
    }
}

impl From<sea_orm::DbErr> for AuthenticationError {
    fn from(e: sea_orm::DbErr) -> Self {
        AuthenticationError::DbError(DbError {
            message: e.to_string(),
        })
    }
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthenticationError::BadCredentials(e) => f.write_str(e.message.as_str()),
            AuthenticationError::DbError(e) => f.write_str(e.message.as_str()),
        }
    }
}

pub async fn get_user(db: &DatabaseConnection, token: &Token) -> Result<Model, AuthenticationError> {
    let user_id = match token.get_user_id() {
        Ok(id) => id,
        Err(e) => {
            tracing::warn!("invalid token");
            return Err(e.into());
        }
    };

    let user = users::find_by_id(user_id).one(db).await?;
    match user {
        Some(u) => Ok(u),
        None => {
            tracing::warn!(user_id = %user_id, "user not found by token");
            Err(AuthenticationError::BadCredentials(BadCredentialsError {
                message: "User not found".to_string(),
            }))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;

    #[tokio::test]
    async fn test_get_user_with_valid_token_returns_user() {
        let db = setup_test_db().await;
        let email = format!("test_valid_{}@example.com", uuid::Uuid::new_v4());
        let user = create_test_user(&db, &email, "hashed_password").await;
        let token = create_test_token(&user);

        let result = get_user(&db, &token).await;

        assert!(result.is_ok());
        let fetched_user = result.unwrap();
        assert_eq!(fetched_user.id, user.id);
        assert_eq!(fetched_user.email, user.email);

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_get_user_with_expired_token_returns_bad_credentials() {
        let db = setup_test_db().await;
        let email = format!("test_expired_{}@example.com", uuid::Uuid::new_v4());
        let user = create_test_user(&db, &email, "hashed_password").await;
        let token = create_expired_token(&user);

        let result = get_user(&db, &token).await;

        assert!(result.is_err());
        match result {
            Err(AuthenticationError::BadCredentials(_)) => {}
            _ => panic!("Expected BadCredentials error"),
        }

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_get_user_with_invalid_signature_returns_bad_credentials() {
        let db = setup_test_db().await;
        let email = format!("test_invalid_sig_{}@example.com", uuid::Uuid::new_v4());
        let user = create_test_user(&db, &email, "hashed_password").await;
        let token = create_invalid_signature_token(&user);

        let result = get_user(&db, &token).await;

        assert!(result.is_err());
        match result {
            Err(AuthenticationError::BadCredentials(_)) => {}
            _ => panic!("Expected BadCredentials error"),
        }

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_get_user_with_nonexistent_user_id_returns_bad_credentials() {
        let db = setup_test_db().await;
        let token = create_token_for_nonexistent_user();

        let result = get_user(&db, &token).await;

        assert!(result.is_err());
        match result {
            Err(AuthenticationError::BadCredentials(e)) => {
                assert_eq!(e.message, "User not found");
            }
            _ => panic!("Expected BadCredentials error"),
        }
    }

    #[tokio::test]
    async fn test_get_user_with_malformed_token_returns_bad_credentials() {
        let db = setup_test_db().await;
        let token = create_malformed_token();

        let result = get_user(&db, &token).await;

        assert!(result.is_err());
        match result {
            Err(AuthenticationError::BadCredentials(_)) => {}
            _ => panic!("Expected BadCredentials error"),
        }
    }
}