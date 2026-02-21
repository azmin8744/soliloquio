use data_access_objects::PostDao;
use models::posts::Model;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

use super::PostRepository;

impl PostRepository {
    pub async fn update_post(
        db: &DatabaseConnection,
        user_id: Uuid,
        id: Uuid,
        title: String,
        content: String,
        is_published: Option<bool>,
    ) -> Result<Model, String> {
        let existing = PostDao::find_by_id_for_user(db, id, user_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Post not found".to_string())?;

        let mut am = existing.into_active_model();
        am.title = ActiveValue::set(title);
        am.markdown_content = ActiveValue::set(Some(content));
        am.updated_at = ActiveValue::set(chrono::Utc::now().naive_utc());

        if let Some(publish) = is_published {
            am.is_published = ActiveValue::set(publish);

            if publish {
                let current = PostDao::find_by_id(db, id)
                    .await
                    .map_err(|e| format!("Database error: {}", e))?
                    .unwrap();
                if current.first_published_at.is_none() {
                    am.first_published_at =
                        ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
                }
            }
        }

        PostDao::update(db, am)
            .await
            .map_err(|e| format!("Database error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::super::PostRepository;
    use crate::test_helpers::*;
    use sea_orm::entity::prelude::Uuid;

    #[tokio::test]
    async fn test_update_post_updates_fields() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_update").await;
        let post = create_test_post(&db, user.id, "Original", "content", false).await;

        let updated = PostRepository::update_post(
            &db, user.id, post.id, "Updated Title".into(), "new content".into(), None,
        ).await.unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.markdown_content.as_deref(), Some("new content"));

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_post_sets_updated_at() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_upd_ts").await;
        let post = create_test_post(&db, user.id, "Title", "content", false).await;
        let original_updated_at = post.updated_at;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated = PostRepository::update_post(
            &db, user.id, post.id, "Updated".into(), "new".into(), None,
        ).await.unwrap();

        assert!(updated.updated_at > original_updated_at);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_post_nonexistent_returns_error() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_upd_404").await;
        let fake_id = Uuid::new_v4();

        let result = PostRepository::update_post(
            &db, user.id, fake_id, "New".into(), "new".into(), None,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_post_publish_sets_first_published_at_once() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_pub_once").await;
        let post = create_test_post(&db, user.id, "Title", "content", false).await;

        let published = PostRepository::update_post(
            &db, user.id, post.id, "Title".into(), "content".into(), Some(true),
        ).await.unwrap();

        let first_pub = published.first_published_at.unwrap();
        assert!(published.is_published);

        PostRepository::update_post(
            &db, user.id, post.id, "Title".into(), "content".into(), Some(false),
        ).await.unwrap();

        let republished = PostRepository::update_post(
            &db, user.id, post.id, "Title".into(), "content".into(), Some(true),
        ).await.unwrap();

        assert_eq!(republished.first_published_at.unwrap(), first_pub);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_post_other_users_post_returns_not_found() {
        let db = setup_test_db().await;
        let (user_a, email_a) = create_test_user(&db, "repo_own_a").await;
        let (user_b, email_b) = create_test_user(&db, "repo_own_b").await;
        let post = create_test_post(&db, user_a.id, "A's Post", "content", false).await;

        let result = PostRepository::update_post(
            &db, user_b.id, post.id, "Hijacked".into(), "evil".into(), None,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        cleanup_user_by_email(&db, &email_a).await;
        cleanup_user_by_email(&db, &email_b).await;
    }
}
