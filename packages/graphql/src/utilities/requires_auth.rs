use std::fmt;
use services::authentication::token::Token;
use async_graphql::{Context, Result};
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
