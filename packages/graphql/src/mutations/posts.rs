use crate::errors::{AuthError, DbError};
use crate::types::post::{DeletedPost, Post as PostType};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, InputObject, Object, Result, Union};
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

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
}

#[derive(InputObject)]
struct UpdatePostInput {
    id: Uuid,
    title: String,
    content: String,
    is_published: Option<bool>,
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
        is_published: p.is_published,
        first_published_at: p.first_published_at,
        created_at: p.created_at,
        updated_at: p.updated_at,
    }
}

#[derive(Default)]
pub struct PostMutation;

trait PostMutations {
    async fn add_post(
        &self,
        ctx: &Context<'_>,
        new_post: AddPostInput,
    ) -> Result<PostMutationResult>;
    async fn update_post(
        &self,
        ctx: &Context<'_>,
        post: UpdatePostInput,
    ) -> Result<PostMutationResult>;
    async fn delete_post(
        &self,
        ctx: &Context<'_>,
        post: DeletePostInput,
    ) -> Result<PostMutationResult>;
}

impl RequiresAuth for PostMutation {}

#[Object]
impl PostMutations for PostMutation {
    async fn add_post(
        &self,
        ctx: &Context<'_>,
        new_post: AddPostInput,
    ) -> Result<PostMutationResult> {
        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let db = ctx.data::<DatabaseConnection>().unwrap();
        let is_published = new_post.is_published.unwrap_or(false);

        match repositories::PostRepository::create_post(
            db,
            user.id,
            new_post.title,
            new_post.content,
            is_published,
        )
        .await
        {
            Ok(p) => Ok(PostMutationResult::ChangedPost(model_to_post_type(&p))),
            Err(e) => {
                tracing::error!("failed to insert post");
                Ok(PostMutationResult::DbError(DbError { message: e }))
            }
        }
    }

    async fn update_post(
        &self,
        ctx: &Context<'_>,
        post: UpdatePostInput,
    ) -> Result<PostMutationResult> {
        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Invalidate cache for this post
        if let Ok(cache) = ctx.data::<crate::utilities::MarkdownCache>() {
            cache.invalidate(&post.id);
        }

        match repositories::PostRepository::update_post(
            db,
            user.id,
            post.id,
            post.title,
            post.content,
            post.is_published,
        )
        .await
        {
            Ok(p) => Ok(PostMutationResult::ChangedPost(model_to_post_type(&p))),
            Err(e) => Ok(PostMutationResult::DbError(DbError { message: e })),
        }
    }

    async fn delete_post(
        &self,
        ctx: &Context<'_>,
        post: DeletePostInput,
    ) -> Result<PostMutationResult> {
        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let db = ctx.data::<DatabaseConnection>().unwrap();

        match repositories::PostRepository::delete_post(db, user.id, post.id).await {
            Ok(id) => Ok(PostMutationResult::DeletedPost(DeletedPost { id })),
            Err(e) => Ok(PostMutationResult::DbError(DbError { message: e })),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

    #[tokio::test]
    async fn test_add_post_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"mutation {
            addPost(newPost: { title: "Test", content: "content" }) {
                ... on AuthError { message }
                ... on Post { id }
            }
        }"#;

        let res = schema.execute(Request::new(query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["addPost"]["message"].as_str().is_some());
    }

    #[tokio::test]
    async fn test_update_post_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_unauth");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let post = create_test_post(&db, user.id, "Title", "content", false).await;

        let query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "New", content: "new" }}) {{
                    ... on AuthError {{ message }}
                    ... on Post {{ id }}
                }}
            }}"#,
            post.id
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["updatePost"]["message"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_delete_post_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("delete_unauth");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let post = create_test_post(&db, user.id, "Title", "content", false).await;

        let query = format!(
            r#"mutation {{
                deletePost(post: {{ id: "{}" }}) {{
                    ... on AuthError {{ message }}
                    ... on DeletedPost {{ id }}
                }}
            }}"#,
            post.id
        );

        let res = schema.execute(Request::new(&query)).await;
        let data = res.data.into_json().unwrap();

        assert!(data["deletePost"]["message"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }
}
