use super::{DeletePostInput, PostMutation, PostMutationResult};
use crate::errors::{AuthError, DbError};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Result};
use sea_orm::*;

pub(super) async fn delete_post(
    mutation: &PostMutation,
    ctx: &Context<'_>,
    post: DeletePostInput,
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

    match repositories::PostRepository::delete_post(db, user.id, post.id).await {
        Ok(id) => Ok(PostMutationResult::DeletedPost(
            crate::types::post::DeletedPost { id },
        )),
        Err(e) => Ok(PostMutationResult::DbError(DbError { message: e })),
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;

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
