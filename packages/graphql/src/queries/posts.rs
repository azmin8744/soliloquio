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

#[cfg(test)]
mod tests {
    use crate::test_helpers::*;
    use async_graphql::Request;
    use services::authentication::Token;

    // ============= posts() query tests =============

    #[tokio::test]
    async fn test_posts_authenticated_returns_user_posts_only() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_auth");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "Post 1", "content 1", false).await;
        create_test_post(&db, user.id, "Post 2", "content 2", true).await;

        let query = r#"query { posts { id title } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        let posts = data["posts"].as_array().unwrap();

        assert_eq!(posts.len(), 2);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let query = r#"query { posts { id } }"#;

        let res = schema.execute(Request::new(query)).await;
        assert!(!res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_posts_empty_returns_empty_vec() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_empty");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let query = r#"query { posts { id } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        let posts = data["posts"].as_array().unwrap();
        assert!(posts.is_empty());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_ordered_by_created_at_desc() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_order");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        // Create posts with slight delay to ensure different timestamps
        create_test_post(&db, user.id, "First", "content", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Second", "content", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Third", "content", false).await;

        let query = r#"query { posts { title } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();
        let posts = data["posts"].as_array().unwrap();

        // Most recent first
        assert_eq!(posts[0]["title"], "Third");
        assert_eq!(posts[1]["title"], "Second");
        assert_eq!(posts[2]["title"], "First");

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_does_not_return_other_users_posts() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let email1 = generate_unique_email("posts_user1");
        let email2 = generate_unique_email("posts_user2");
        let user1 = create_test_user_with_password(&db, &email1, &valid_password()).await;
        let user2 = create_test_user_with_password(&db, &email2, &valid_password()).await;
        let token1 = create_access_token(&user1);

        create_test_post(&db, user1.id, "User1 Post", "content", false).await;
        create_test_post(&db, user2.id, "User2 Post", "content", false).await;

        let query = r#"query { posts { title } }"#;

        let res = schema
            .execute(Request::new(query).data(Token::new(token1)))
            .await;
        let data = res.data.into_json().unwrap();
        let posts = data["posts"].as_array().unwrap();

        assert_eq!(posts.len(), 1);
        assert_eq!(posts[0]["title"], "User1 Post");

        cleanup_test_user_by_email(&db, &email1).await;
        cleanup_test_user_by_email(&db, &email2).await;
    }

    // ============= post(id) query tests =============

    #[tokio::test]
    async fn test_post_authenticated_returns_own_post() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("post_own");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let post = create_test_post(&db, user.id, "My Post", "my content", true).await;

        let query = format!(r#"query {{ post(id: "{}") {{ id title }} }}"#, post.id);

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["post"]["title"], "My Post");

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_post_unauthenticated_returns_auth_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let fake_id = uuid::Uuid::new_v4();

        let query = format!(r#"query {{ post(id: "{}") {{ id }} }}"#, fake_id);

        let res = schema.execute(Request::new(&query)).await;
        assert!(!res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_post_nonexistent_id_returns_none() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("post_nonexist");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let fake_id = uuid::Uuid::new_v4();

        let query = format!(r#"query {{ post(id: "{}") {{ id }} }}"#, fake_id);

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["post"].is_null());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_post_other_users_post_returns_none() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let email1 = generate_unique_email("post_other1");
        let email2 = generate_unique_email("post_other2");
        let user1 = create_test_user_with_password(&db, &email1, &valid_password()).await;
        let user2 = create_test_user_with_password(&db, &email2, &valid_password()).await;
        let token1 = create_access_token(&user1);

        // user2's post
        let post = create_test_post(&db, user2.id, "User2 Post", "content", false).await;

        // user1 tries to access it
        let query = format!(r#"query {{ post(id: "{}") {{ id }} }}"#, post.id);

        let res = schema
            .execute(Request::new(&query).data(Token::new(token1)))
            .await;
        let data = res.data.into_json().unwrap();

        assert!(data["post"].is_null());

        cleanup_test_user_by_email(&db, &email1).await;
        cleanup_test_user_by_email(&db, &email2).await;
    }

    #[tokio::test]
    async fn test_post_returns_all_fields() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("post_fields");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);
        let post = create_test_post(&db, user.id, "Full Post", "# Content", true).await;

        let query = format!(
            r#"query {{ post(id: "{}") {{
                id title markdownContent isPublished firstPublishedAt createdAt
            }} }}"#,
            post.id
        );

        let res = schema
            .execute(Request::new(&query).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["post"]["id"], post.id.to_string());
        assert_eq!(data["post"]["title"], "Full Post");
        assert_eq!(data["post"]["markdownContent"], "# Content");
        assert_eq!(data["post"]["isPublished"], true);
        assert!(data["post"]["firstPublishedAt"].as_str().is_some());
        assert!(data["post"]["createdAt"].as_str().is_some());

        cleanup_test_user_by_email(&db, &email).await;
    }
}