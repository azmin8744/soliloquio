use crate::authentication::token::Token;
use std::fmt;
use sea_orm::*;
use models::users::{Model, Entity as users};

pub struct BadCredentialsError {
    pub message: String,
}

pub struct DbError {
    pub message: String,
}

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
    let user_id = token.get_user_id()?;

    let user = users::find_by_id(user_id).one(db).await?;
    if user.is_none() {
        return Err(AuthenticationError::BadCredentials(BadCredentialsError {
            message: "User not found".to_string(),
        }));
    }
    Ok(user.unwrap())
}