use super::{AddPostInput, PostMutation, PostMutationResult, model_to_post_type};
use crate::errors::{AuthError, DbError};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result};
use sea_orm::*;

pub(super) async fn add_post(
    mutation: &PostMutation,
    ctx: &Context<'_>,
    new_post: AddPostInput,
) -> Result<PostMutationResult> {
    let user = match mutation.require_authenticate_as_user(ctx).await {
        Ok(user) => user,
        Err(e) => {
            return Ok(PostMutationResult::AuthError(AuthError {
                message: e.to_string(),
            }));
        }
    };
    if user.email_verified_at.is_none() {
        return Ok(PostMutationResult::AuthError(AuthError {
            message: "Email not verified".to_string(),
        }));
    }

    let db = ctx.data::<DatabaseConnection>().unwrap();
    let is_published = new_post.is_published.unwrap_or(false);

    match repositories::PostRepository::create_post(
        db,
        user.id,
        new_post.title,
        new_post.content,
        is_published,
        new_post.description,
        new_post.slug,
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
}
