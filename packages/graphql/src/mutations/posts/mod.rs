use crate::errors::{AuthError, DbError};
use crate::types::post::{DeletedPost, Post as PostType};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, InputObject, Object, Result, Union};
use sea_orm::entity::prelude::Uuid;

mod add_post;
mod delete_post;
mod update_post;

#[derive(Union)]
pub enum PostMutationResult {
    ChangedPost(PostType),
    DeletedPost(DeletedPost),
    DbError(DbError),
    AuthError(AuthError),
}

#[derive(InputObject)]
struct AddPostInput {
    title: String,
    content: String,
    is_published: Option<bool>,
    description: Option<String>,
    slug: Option<String>,
}

#[derive(InputObject)]
struct UpdatePostInput {
    id: Uuid,
    title: String,
    content: String,
    is_published: Option<bool>,
    description: Option<String>,
    slug: Option<String>,
}

#[derive(InputObject)]
struct DeletePostInput {
    id: Uuid,
}

fn model_to_post_type(p: &models::posts::Model) -> PostType {
    PostType {
        id: p.id,
        title: p.title.clone(),
        markdown_content: p.markdown_content.clone().unwrap_or_default(),
        description: p.description.clone(),
        slug: p.slug.clone(),
        is_published: p.is_published,
        first_published_at: p.first_published_at,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

#[derive(Default)]
pub struct PostMutation;

impl RequiresAuth for PostMutation {}

#[Object]
impl PostMutation {
    async fn add_post(
        &self,
        ctx: &Context<'_>,
        new_post: AddPostInput,
    ) -> Result<PostMutationResult> {
        add_post::add_post(self, ctx, new_post).await
    }

    async fn update_post(
        &self,
        ctx: &Context<'_>,
        post: UpdatePostInput,
    ) -> Result<PostMutationResult> {
        update_post::update_post(self, ctx, post).await
    }

    async fn delete_post(
        &self,
        ctx: &Context<'_>,
        post: DeletePostInput,
    ) -> Result<PostMutationResult> {
        delete_post::delete_post(self, ctx, post).await
    }
}
