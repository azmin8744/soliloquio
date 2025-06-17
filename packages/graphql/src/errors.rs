use async_graphql::SimpleObject;
use std::fmt;
use services::AuthenticationError;

#[derive(SimpleObject, Debug)]
pub struct DbErr {
    pub message: String,
}

impl From<sea_orm::error::DbErr> for DbErr {
    fn from(e: sea_orm::error::DbErr) -> Self {
        DbErr { message: e.to_string() }
    }
}

impl fmt::Display for DbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

#[derive(SimpleObject, Debug)]
pub struct AuthError {
    pub message: String,
}

impl From<AuthenticationError> for AuthError {
    fn from(e: AuthenticationError) -> Self {
        AuthError { message: e.to_string() }
    }
}

impl From<crate::utilities::requires_auth::AuthenticationError> for AuthError {
    fn from(e: crate::utilities::requires_auth::AuthenticationError) -> Self {
        AuthError { message: e.to_string() }
    }
}

impl From<sea_orm::error::DbErr> for AuthError {
    fn from(e: sea_orm::error::DbErr) -> Self {
        AuthError { message: e.to_string() }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

#[derive(SimpleObject, Debug)]
pub struct ValidationErrorType {
    pub message: String,
}
