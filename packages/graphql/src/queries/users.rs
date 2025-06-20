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
