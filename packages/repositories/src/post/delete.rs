use data_access_objects::PostDao;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

use super::PostRepository;

impl PostRepository {
    pub async fn delete_post(
        db: &DatabaseConnection,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Uuid, String> {
        let existing = PostDao::find_by_id_for_user(db, id, user_id)
            .await
            .map_err(|e| format!("Database error: {}", e))?
            .ok_or_else(|| "Post not found".to_string())?;

        let am = existing.into_active_model();
        PostDao::delete(db, am)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        Ok(id)
    }
}

#[cfg(test)]
mod tests {
    use super::super::PostRepository;
    use crate::test_helpers::*;
    use data_access_objects::PostDao;
    use sea_orm::entity::prelude::Uuid;

    #[tokio::test]
    async fn test_delete_post_deletes() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_del").await;
        let post = create_test_post(&db, user.id, "To Delete", "content", false).await;

        let id = PostRepository::delete_post(&db, user.id, post.id).await.unwrap();
        assert_eq!(id, post.id);

        let found = PostDao::find_by_id(&db, post.id).await.unwrap();
        assert!(found.is_none());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_delete_post_nonexistent_returns_error() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_del_404").await;
        let fake_id = Uuid::new_v4();

        let result = PostRepository::delete_post(&db, user.id, fake_id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_delete_post_other_users_post_returns_not_found() {
        let db = setup_test_db().await;
        let (user_a, email_a) = create_test_user(&db, "repo_del_own_a").await;
        let (user_b, email_b) = create_test_user(&db, "repo_del_own_b").await;
        let post = create_test_post(&db, user_a.id, "A's Post", "content", false).await;

        let result = PostRepository::delete_post(&db, user_b.id, post.id).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));

        let found = PostDao::find_by_id(&db, post.id).await.unwrap();
        assert!(found.is_some());

        cleanup_user_by_email(&db, &email_a).await;
        cleanup_user_by_email(&db, &email_b).await;
    }
}
