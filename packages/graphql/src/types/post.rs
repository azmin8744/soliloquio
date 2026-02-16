use crate::utilities::markdown::{render_markdown_cached, MarkdownCache};
use async_graphql::{Context, Object, SimpleObject};
use chrono::NaiveDateTime;
use uuid::Uuid;

pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub markdown_content: String, // The original markdown
    pub is_published: bool,
    pub first_published_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

#[Object]
impl Post {
    async fn id(&self) -> Uuid {
        self.id
    }

    async fn title(&self) -> &String {
        &self.title
    }

    async fn is_published(&self) -> bool {
        self.is_published
    }

    async fn first_published_at(&self) -> Option<NaiveDateTime> {
        self.first_published_at
    }

    async fn created_at(&self) -> NaiveDateTime {
        self.created_at
    }

    async fn updated_at(&self) -> NaiveDateTime {
        self.updated_at
    }

    /// Returns the original markdown content for editing
    async fn markdown_content(&self) -> &String {
        &self.markdown_content
    }

    /// Returns the rendered HTML content for display
    async fn content(&self, ctx: &Context<'_>) -> String {
        let default_cache = MarkdownCache::default();
        let cache = ctx.data::<MarkdownCache>().unwrap_or(&default_cache);
        render_markdown_cached(self.id, &self.markdown_content, cache)
    }
}

#[derive(SimpleObject)]
pub struct DeletedPost {
    pub id: Uuid,
}
