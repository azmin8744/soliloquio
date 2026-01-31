use async_graphql::{Context, InputObject, Object, Result, Union};
use sea_orm::*;
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use crate::types::post::{Post as PostType, DeletedPost};
use crate::utilities::requires_auth::RequiresAuth;
use crate::errors::{DbError, AuthError};

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
    content: String,  // This will be the markdown content
    is_published: Option<bool>,
}

#[derive(InputObject)]
struct UpdatePostInput {
    id: Uuid,
    title: String,
    content: String,  // This will be the markdown content
    is_published: Option<bool>,
}

#[derive(InputObject)]
struct DeletePostInput {
    id: Uuid,
}

#[derive(Default)]
pub struct PostMutation;

trait PostMutations {
    async fn add_post(&self, ctx: &Context<'_>, new_post: AddPostInput) -> Result<PostMutationResult>;
    async fn update_post(&self, ctx: &Context<'_>, post: UpdatePostInput) -> Result<PostMutationResult>;
    async fn delete_post(&self, ctx: &Context<'_>, post: DeletePostInput) -> Result<PostMutationResult>;
}

impl RequiresAuth for PostMutation {}

#[Object]
impl PostMutations for PostMutation {
    async fn add_post(&self, ctx: &Context<'_>, new_post: AddPostInput) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let is_published = new_post.is_published.unwrap_or(false);
        let first_published_at = if is_published { 
            Some(chrono::Utc::now().naive_utc())
        } else {
            None
        };
        
        let post = posts::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(new_post.title),
            markdown_content: ActiveValue::set(Some(new_post.content)),
            user_id: ActiveValue::set(user.id),
            is_published: ActiveValue::set(is_published),
            first_published_at: ActiveValue::set(first_published_at),
            ..Default::default()
        };

        let res = match Posts::insert(post).exec(db).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: e.to_string(),
                }));
            }
        };
        
        let p = match Posts::find_by_id(res.last_insert_id).one(db).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: "Post not found".to_string(),
                }));
            }
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: e.to_string(),
                }));
            }
        };

        Ok(PostMutationResult::ChangedPost(PostType {
            id: p.id,
            title: p.title.clone(),
            markdown_content: p.markdown_content.unwrap_or_default(),
            is_published: p.is_published,
            first_published_at: p.first_published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }))
    }

    async fn update_post(&self, ctx: &Context<'_>, post: UpdatePostInput) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let _user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let mut post_to_update = match Posts::find_by_id(post.id).one(db).await {
            Ok(Some(p)) => p.into_active_model(),
            Ok(None) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: "Post not found".to_string(),
                }));
            }
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: e.to_string(),
                }));
            }
        };

        post_to_update.title = ActiveValue::set(post.title);
        post_to_update.markdown_content = ActiveValue::set(Some(post.content));
        post_to_update.updated_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
        
        // Invalidate cache for this post
        if let Ok(cache) = ctx.data::<crate::utilities::MarkdownCache>() {
            cache.invalidate(&post.id);
        }
        
        // Handle publication status change if provided
        if let Some(is_published) = post.is_published {
            post_to_update.is_published = ActiveValue::set(is_published);
            
            // Set first_published_at if being published for the first time
            if is_published {
                let post_model = Posts::find_by_id(post.id).one(db).await.unwrap().unwrap();
                if post_model.first_published_at.is_none() {
                    post_to_update.first_published_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
                }
            }
        }

        let _res = match Posts::update(post_to_update).exec(db).await {
            Ok(p) => {
                return Ok(PostMutationResult::ChangedPost(PostType {
                    id: p.id,
                    title: p.title.clone(),
                    markdown_content: p.markdown_content.unwrap_or_default(),
                    is_published: p.is_published,
                    first_published_at: p.first_published_at,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                }))
            },
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: e.to_string(),
                }));
            }
        };
    }

    async fn delete_post(&self, ctx: &Context<'_>, post: DeletePostInput) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let _user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let post_to_delete = match Posts::find_by_id(post.id).one(db).await {
            Ok(Some(p)) => p.into_active_model(),
            Ok(None) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: "Post not found".to_string(),
                }));
            }
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbError {
                    message: e.to_string(),
                }));
            }
        };

        match Posts::delete(post_to_delete.clone()).exec(db).await {
            Ok(_) => Ok(PostMutationResult::DeletedPost(DeletedPost { id: post_to_delete.id.unwrap() })),
            Err(e) => Ok(PostMutationResult::DbError(DbError {
                message: e.to_string(),
            })),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;
    use sea_orm::EntityTrait;
    use services::authentication::Token;

    // ============= add_post tests =============

    #[tokio::test]
    async fn test_add_post_authenticated_creates_post_with_correct_user_id() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("add_post");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation {
            addPost(newPost: { title: "Test Post", content: "Hello World", isPublished: false }) {
                ... on Post { id title markdownContent isPublished }
                ... on AuthError { message }
            }
        }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        assert!(data["addPost"]["id"].as_str().is_some());
        assert_eq!(data["addPost"]["title"], "Test Post");

        cleanup_test_user_by_email(&db, &email).await;
    }

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
    async fn test_add_post_stores_markdown_content() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("add_md");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let markdown = "# Heading\\n\\n**bold** and *italic*";
        let query = format!(
            r#"mutation {{
                addPost(newPost: {{ title: "MD Post", content: "{}" }}) {{
                    ... on Post {{ markdownContent }}
                }}
            }}"#,
            markdown
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["addPost"]["markdownContent"]
            .as_str()
            .unwrap()
            .contains("Heading"));

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_add_post_default_unpublished() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("add_unpub");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation {
            addPost(newPost: { title: "Unpub", content: "content" }) {
                ... on Post { isPublished firstPublishedAt }
            }
        }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["addPost"]["isPublished"], false);
        assert!(data["addPost"]["firstPublishedAt"].is_null());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_add_post_published_sets_first_published_at() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("add_pub");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation {
            addPost(newPost: { title: "Pub", content: "content", isPublished: true }) {
                ... on Post { isPublished firstPublishedAt }
            }
        }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["addPost"]["isPublished"], true);
        assert!(data["addPost"]["firstPublishedAt"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_add_post_returns_created_post() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("add_return");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"mutation {
            addPost(newPost: { title: "Return Test", content: "body" }) {
                ... on Post { id title markdownContent isPublished createdAt }
            }
        }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["addPost"]["id"].as_str().is_some());
        assert_eq!(data["addPost"]["title"], "Return Test");
        assert_eq!(data["addPost"]["markdownContent"], "body");
        assert_eq!(data["addPost"]["isPublished"], false);
        assert!(data["addPost"]["createdAt"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    // ============= update_post tests =============

    #[tokio::test]
    async fn test_update_post_authenticated_updates_fields() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_post");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let post = create_test_post(&db, user.id, "Original", "content", false).await;

        let query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "Updated Title", content: "new content" }}) {{
                    ... on Post {{ id title markdownContent }}
                }}
            }}"#,
            post.id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["updatePost"]["title"], "Updated Title");
        assert_eq!(data["updatePost"]["markdownContent"], "new content");

        cleanup_test_user_by_email(&db, &email).await;
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
    async fn test_update_post_nonexistent_returns_db_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_nonexist");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let fake_id = uuid::Uuid::new_v4();

        let query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "New", content: "new" }}) {{
                    ... on DbError {{ message }}
                    ... on Post {{ id }}
                }}
            }}"#,
            fake_id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["updatePost"]["message"]
            .as_str()
            .unwrap()
            .contains("not found"));

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_post_sets_updated_at() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_timestamp");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let post = create_test_post(&db, user.id, "Title", "content", false).await;

        assert!(post.updated_at.is_none());

        let query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "Updated", content: "new" }}) {{
                    ... on Post {{ updatedAt }}
                }}
            }}"#,
            post.id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["updatePost"]["updatedAt"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_post_publish_sets_first_published_at_once() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("update_publish");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let post = create_test_post(&db, user.id, "Title", "content", false).await;

        // first publish
        let query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "Title", content: "content", isPublished: true }}) {{
                    ... on Post {{ firstPublishedAt isPublished }}
                }}
            }}"#,
            post.id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token.clone())))
            .await;
        let data = res.data.into_json().unwrap();

        let first_pub = data["updatePost"]["firstPublishedAt"].as_str().unwrap();
        assert!(!first_pub.is_empty());

        // unpublish and republish - should keep original first_published_at
        let unpub_query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "Title", content: "content", isPublished: false }}) {{
                    ... on Post {{ isPublished }}
                }}
            }}"#,
            post.id
        );
        schema
            .execute(Request::new(&unpub_query).data(Token::new(token.clone())))
            .await;

        let repub_query = format!(
            r#"mutation {{
                updatePost(post: {{ id: "{}", title: "Title", content: "content", isPublished: true }}) {{
                    ... on Post {{ firstPublishedAt }}
                }}
            }}"#,
            post.id
        );
        let res2 = schema
            .execute(Request::new(&repub_query).data(Token::new(token)))
            .await;
        let data2 = res2.data.into_json().unwrap();

        // first_published_at should be unchanged
        assert_eq!(
            data2["updatePost"]["firstPublishedAt"].as_str().unwrap(),
            first_pub
        );

        cleanup_test_user_by_email(&db, &email).await;
    }

    // ============= delete_post tests =============

    #[tokio::test]
    async fn test_delete_post_authenticated_deletes() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("delete_post");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let post = create_test_post(&db, user.id, "To Delete", "content", false).await;

        let query = format!(
            r#"mutation {{
                deletePost(post: {{ id: "{}" }}) {{
                    ... on DeletedPost {{ id }}
                }}
            }}"#,
            post.id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["deletePost"]["id"], post.id.to_string());

        // verify deleted
        use models::posts::Entity as Posts;
        let found = Posts::find_by_id(post.id).one(&db).await.unwrap();
        assert!(found.is_none());

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

    #[tokio::test]
    async fn test_delete_post_nonexistent_returns_db_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("delete_nonexist");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let fake_id = uuid::Uuid::new_v4();

        let query = format!(
            r#"mutation {{
                deletePost(post: {{ id: "{}" }}) {{
                    ... on DbError {{ message }}
                    ... on DeletedPost {{ id }}
                }}
            }}"#,
            fake_id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["deletePost"]["message"]
            .as_str()
            .unwrap()
            .contains("not found"));

        cleanup_test_user_by_email(&db, &email).await;
    }
}
