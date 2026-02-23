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
        description: Option<String>,
        slug: Option<String>,
    ) -> Result<posts::Model, String> {
        let first_published_at = if is_published {
            Some(chrono::Utc::now().naive_utc())
        } else {
            None
        };

        match slug.filter(|s| !s.is_empty()) {
            None => {
                let model = posts::ActiveModel {
                    id: ActiveValue::set(Uuid::new_v4()),
                    title: ActiveValue::set(title),
                    markdown_content: ActiveValue::set(Some(content)),
                    user_id: ActiveValue::set(user_id),
                    is_published: ActiveValue::set(is_published),
                    first_published_at: ActiveValue::set(first_published_at),
                    description: ActiveValue::set(description),
                    slug: ActiveValue::set(None),
                    ..Default::default()
                };
                PostDao::insert(db, model)
                    .await
                    .map_err(|e| format!("Database error: {}", e))
            }
            Some(base_slug) => {
                for attempt in 0u32..10 {
                    let candidate = if attempt == 0 {
                        base_slug.clone()
                    } else {
                        format!("{}-{}", base_slug, attempt + 1)
                    };

                    let model = posts::ActiveModel {
                        id: ActiveValue::set(Uuid::new_v4()),
                        title: ActiveValue::set(title.clone()),
                        markdown_content: ActiveValue::set(Some(content.clone())),
                        user_id: ActiveValue::set(user_id),
                        is_published: ActiveValue::set(is_published),
                        first_published_at: ActiveValue::set(first_published_at),
                        description: ActiveValue::set(description.clone()),
                        slug: ActiveValue::set(Some(candidate)),
                        ..Default::default()
                    };

                    match PostDao::insert(db, model).await {
                        Ok(post) => return Ok(post),
                        Err(e) => {
                            let msg = e.to_string();
                            if msg.contains("23505")
                                || msg.contains("duplicate key")
                                || msg.contains("unique constraint")
                            {
                                continue;
                            }
                            return Err(format!("Database error: {}", e));
                        }
                    }
                }
                Err("Could not generate unique slug after 10 attempts".to_string())
            }
        }
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
            &db, user.id, "MD Post".into(), "# Heading".into(), false, None, None,
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
            &db, user.id, "Unpub".into(), "c".into(), false, None, None,
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
            &db, user.id, "Pub".into(), "c".into(), true, None, None,
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
            &db, user.id, "Return Test".into(), "body".into(), false, None, None,
        ).await.unwrap();

        assert!(!post.id.is_nil());
        assert_eq!(post.title, "Return Test");
        assert_eq!(post.markdown_content.as_deref(), Some("body"));
        assert!(!post.is_published);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_create_post_null_slug_when_not_provided() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_slug_null").await;

        let post = PostRepository::create_post(
            &db, user.id, "Hello World".into(), "".into(), false, None, None,
        ).await.unwrap();

        assert!(post.slug.is_none());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_create_post_uses_provided_slug() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_slug_custom").await;

        let post = PostRepository::create_post(
            &db, user.id, "Title".into(), "".into(), false, None, Some("my-custom-slug".into()),
        ).await.unwrap();

        assert_eq!(post.slug.as_deref(), Some("my-custom-slug"));

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_create_post_deduplicates_slug() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_slug_dedup").await;

        let p1 = PostRepository::create_post(
            &db, user.id, "P1".into(), "".into(), false, None, Some("my-slug".into()),
        ).await.unwrap();
        let p2 = PostRepository::create_post(
            &db, user.id, "P2".into(), "".into(), false, None, Some("my-slug".into()),
        ).await.unwrap();

        assert_ne!(p1.slug, p2.slug);
        assert_eq!(p1.slug.as_deref(), Some("my-slug"));
        assert_eq!(p2.slug.as_deref(), Some("my-slug-2"));

        cleanup_user_by_email(&db, &email).await;
    }
}
