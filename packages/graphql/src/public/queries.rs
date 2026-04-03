use super::types::{PublicAuthor, PublicPost};
use crate::types::sort::SortDirection;
use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::{Context, Enum, Object, Result, SimpleObject};
use models::{posts, users};
use repositories::PostRepository;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;
use services::api_keys as api_key_service;

#[derive(Enum, Copy, Clone, Eq, PartialEq, Debug)]
pub enum PublicPostSortBy {
    #[graphql(name = "CREATED_AT")]
    CreatedAt,
    #[graphql(name = "UPDATED_AT")]
    UpdatedAt,
    #[graphql(name = "TITLE")]
    Title,
    #[graphql(name = "FIRST_PUBLISHED_AT")]
    FirstPublishedAt,
}

impl From<PublicPostSortBy> for repositories::PostSortBy {
    fn from(v: PublicPostSortBy) -> Self {
        match v {
            PublicPostSortBy::CreatedAt => Self::CreatedAt,
            PublicPostSortBy::UpdatedAt => Self::UpdatedAt,
            PublicPostSortBy::Title => Self::Title,
            PublicPostSortBy::FirstPublishedAt => Self::FirstPublishedAt,
        }
    }
}

#[derive(SimpleObject, Default)]
struct PostConnectionExtra {
    page_number: Option<i32>,
    total_pages: Option<i32>,
}

pub struct PublicApiKey(pub String);

async fn require_user(ctx: &Context<'_>) -> Result<Uuid, async_graphql::Error> {
    let api_key_str = ctx
        .data::<PublicApiKey>()
        .map_err(|_| async_graphql::Error::new("Missing API key"))?;
    let db = ctx.data::<DatabaseConnection>().unwrap();
    api_key_service::validate(db, &api_key_str.0)
        .await
        .ok_or_else(|| async_graphql::Error::new("Invalid API key"))
}

fn model_to_public_post(p: &models::posts::Model) -> PublicPost {
    PublicPost {
        id: p.id,
        user_id: p.user_id,
        title: p.title.clone(),
        description: p.description.clone(),
        slug: p.slug.clone(),
        cover_image: p.cover_image.clone(),
        markdown_content: p.markdown_content.clone().unwrap_or_default(),
        first_published_at: p.first_published_at,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

#[derive(Default)]
pub struct PublicQueryRoot;

#[Object]
impl PublicQueryRoot {
    #[graphql(complexity = "first.unwrap_or(10) as usize * child_complexity")]
    async fn posts(
        &self,
        ctx: &Context<'_>,
        page: Option<i32>,
        first: Option<i32>,
        sort_by: Option<PublicPostSortBy>,
        sort_direction: Option<SortDirection>,
        search: Option<String>,
    ) -> Result<Connection<String, PublicPost, PostConnectionExtra, EmptyFields>> {
        let user_id = require_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let search = search.as_deref().map(str::trim).filter(|s| !s.is_empty());
        if let Some(q) = search {
            let all_posts = PostRepository::search_posts(db, user_id, q)
                .await
                .map_err(|e| async_graphql::Error::new(e))?;
            let mut conn = Connection::with_additional_fields(false, false, PostConnectionExtra::default());
            for post in all_posts.iter().filter(|p| p.is_published) {
                conn.edges.push(Edge::new(post.id.to_string(), model_to_public_post(post)));
            }
            return Ok(conn);
        }

        let sort_by = sort_by.unwrap_or(PublicPostSortBy::CreatedAt);
        let sort_dir = sort_direction.unwrap_or(SortDirection::Desc);

        let result = PostRepository::get_published_posts(
            db,
            user_id,
            page,
            first,
            sort_by.into(),
            sort_dir.into(),
        )
        .await
        .map_err(|e| async_graphql::Error::new(e))?;

        let extra = PostConnectionExtra {
            page_number: result.page_number.map(|n| n as i32),
            total_pages: result.total_pages.map(|n| n as i32),
        };
        let mut conn = Connection::with_additional_fields(result.has_previous_page, result.has_next_page, extra);
        for post in &result.posts {
            conn.edges.push(Edge::new(post.id.to_string(), model_to_public_post(post)));
        }
        Ok(conn)
    }

    async fn post(
        &self,
        ctx: &Context<'_>,
        id: Option<Uuid>,
        slug: Option<String>,
    ) -> Result<Option<PublicPost>> {
        let user_id = require_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let post = if let Some(post_id) = id {
            posts::Entity::find_by_id(post_id)
                .filter(posts::Column::UserId.eq(user_id))
                .filter(posts::Column::IsPublished.eq(true))
                .one(db)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
        } else if let Some(s) = slug {
            posts::Entity::find()
                .filter(posts::Column::UserId.eq(user_id))
                .filter(posts::Column::Slug.eq(s))
                .filter(posts::Column::IsPublished.eq(true))
                .one(db)
                .await
                .map_err(|e| async_graphql::Error::new(e.to_string()))?
        } else {
            return Err(async_graphql::Error::new("Provide id or slug"));
        };

        Ok(post.as_ref().map(model_to_public_post))
    }

    async fn author(&self, ctx: &Context<'_>) -> Result<PublicAuthor> {
        let user_id = require_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let user = users::Entity::find_by_id(user_id)
            .one(db)
            .await
            .map_err(|e| async_graphql::Error::new(e.to_string()))?
            .ok_or_else(|| async_graphql::Error::new("User not found"))?;
        Ok(PublicAuthor {
            id: user.id,
            display_name: user.display_name,
            bio: user.bio,
        })
    }
}
