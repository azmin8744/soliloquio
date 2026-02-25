use super::{PostMutation, PostMutationResult, UpdatePostInput, model_to_post_type};
use crate::errors::{AuthError, DbError};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result};
use sea_orm::*;

pub(super) async fn update_post(
    mutation: &PostMutation,
    ctx: &Context<'_>,
    post: UpdatePostInput,
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
        post.description,
        post.slug,
    )
    .await
    {
        Ok(p) => Ok(PostMutationResult::ChangedPost(model_to_post_type(&p))),
        Err(e) => Ok(PostMutationResult::DbError(DbError { message: e })),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

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
}
