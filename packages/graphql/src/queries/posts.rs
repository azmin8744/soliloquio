use async_graphql::{Context, Object, Result};
use sea_orm::*;
use models::prelude::*;
use sea_orm::entity::prelude::Uuid;
use crate::types::post::Post as PostType;
use crate::utilities::requires_auth::RequiresAuth;
use crate::errors::AuthError;

#[derive(Default)]
pub struct PostQueries;

impl RequiresAuth for PostQueries {}

#[Object]
impl PostQueries {
    /// Get all posts for the authenticated user
    async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<PostType>, AuthError> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();
        
        let posts = Posts::find()
            .filter(models::posts::Column::UserId.eq(user.id))
            .order_by_desc(models::posts::Column::CreatedAt)
            .all(db)
            .await
            .map_err(|e| AuthError {
                message: format!("Database error: {}", e)
            })?;

        let mut result: Vec<PostType> = Vec::new();
        for post in posts {
            let p = PostType {
                id: post.id,
                title: post.title,
                markdown_content: post.markdown_content.unwrap_or_default(),
                is_published: post.is_published,
                first_published_at: post.first_published_at,
                created_at: post.created_at,
                updated_at: post.updated_at,
            };
            result.push(p);
        }
        
        Ok(result)
    }

    /// Get a specific post by ID for the authenticated user
    async fn post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<PostType>, AuthError> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();
        
        let post = Posts::find_by_id(id)
            .filter(models::posts::Column::UserId.eq(user.id))
            .one(db)
            .await
            .map_err(|e| AuthError {
                message: format!("Database error: {}", e)
            })?;

        if let Some(post) = post {
            let p = PostType {
                id: post.id,
                title: post.title,
                markdown_content: post.markdown_content.unwrap_or_default(),
                is_published: post.is_published,
                first_published_at: post.first_published_at,
                created_at: post.created_at,
                updated_at: post.updated_at,
            };
            Ok(Some(p))
        } else {
            Ok(None)
        }
    }
}