use crate::utilities::markdown::{render_markdown_cached, MarkdownCache};
use async_graphql::{Context, Object, Result, SimpleObject};
use chrono::NaiveDateTime;
use models::users;
use sea_orm::*;
use uuid::Uuid;

pub struct PublicPost {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub slug: Option<String>,
    pub markdown_content: String,
    pub first_published_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[Object]
impl PublicPost {
    async fn id(&self) -> Uuid { self.id }
    async fn title(&self) -> &str { &self.title }
    async fn description(&self) -> Option<&str> { self.description.as_deref() }
    async fn slug(&self) -> Option<&str> { self.slug.as_deref() }
    async fn first_published_at(&self) -> Option<NaiveDateTime> { self.first_published_at }
    async fn created_at(&self) -> NaiveDateTime { self.created_at }
    async fn updated_at(&self) -> NaiveDateTime { self.updated_at }

    /// Rendered HTML content
    #[graphql(complexity = 5)]
    async fn content(&self, ctx: &Context<'_>) -> String {
        let default_cache = MarkdownCache::default();
        let cache = ctx.data::<MarkdownCache>().unwrap_or(&default_cache);
        render_markdown_cached(self.id, &self.markdown_content, cache)
    }

    #[graphql(complexity = 3)]
    async fn author(&self, ctx: &Context<'_>) -> Result<PublicAuthor> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let user = users::Entity::find_by_id(self.user_id)
            .one(db)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("Author not found"))?;
        Ok(PublicAuthor { id: user.id, display_name: user.display_name, bio: user.bio })
    }
}

#[derive(SimpleObject)]
pub struct PublicAuthor {
    pub id: Uuid,
    pub display_name: Option<String>,
    pub bio: Option<String>,
}
