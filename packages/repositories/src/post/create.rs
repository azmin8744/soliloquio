use data_access_objects::PostDao;
use models::posts;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

use super::PostRepository;

impl PostRepository {
    pub async fn create_post(
        db: &DatabaseConnection,
        user_id: Uuid,
        title: String,
        content: String,
        is_published: bool,
    ) -> Result<posts::Model, String> {
        let first_published_at = if is_published {
            Some(chrono::Utc::now().naive_utc())
        } else {
            None
        };

        let model = posts::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(title),
            markdown_content: ActiveValue::set(Some(content)),
            user_id: ActiveValue::set(user_id),
            is_published: ActiveValue::set(is_published),
            first_published_at: ActiveValue::set(first_published_at),
            ..Default::default()
        };

        PostDao::insert(db, model)
            .await
            .map_err(|e| format!("Database error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::super::PostRepository;
    use crate::test_helpers::*;

    #[tokio::test]
    async fn test_create_post_stores_content() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_create").await;

        let post = PostRepository::create_post(
            &db, user.id, "MD Post".into(), "# Heading".into(), false,
        ).await.unwrap();

        assert_eq!(post.title, "MD Post");
        assert_eq!(post.markdown_content.as_deref(), Some("# Heading"));

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_create_post_default_unpublished() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_unpub").await;

        let post = PostRepository::create_post(
            &db, user.id, "Unpub".into(), "c".into(), false,
        ).await.unwrap();

        assert!(!post.is_published);
        assert!(post.first_published_at.is_none());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_create_post_published_sets_first_published_at() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_pub").await;

        let post = PostRepository::create_post(
            &db, user.id, "Pub".into(), "c".into(), true,
        ).await.unwrap();

        assert!(post.is_published);
        assert!(post.first_published_at.is_some());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_create_post_returns_all_fields() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_fields").await;

        let post = PostRepository::create_post(
            &db, user.id, "Return Test".into(), "body".into(), false,
        ).await.unwrap();

        assert!(!post.id.is_nil());
        assert_eq!(post.title, "Return Test");
        assert_eq!(post.markdown_content.as_deref(), Some("body"));
        assert!(!post.is_published);

        cleanup_user_by_email(&db, &email).await;
    }
}
