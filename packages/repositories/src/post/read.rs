use data_access_objects::PostDao;
use models::posts::Model;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

use super::{
    build_keyset_filter, decode_cursor, encode_cursor, sort_column, PaginatedPosts, PostRepository,
    PostSortBy, SortDirection, DEFAULT_PAGE_SIZE, MAX_PAGE_SIZE,
};

impl PostRepository {
    pub async fn get_posts(
        db: &DatabaseConnection,
        user_id: Uuid,
        after: Option<&str>,
        first: Option<i32>,
        sort_by: PostSortBy,
        sort_dir: SortDirection,
    ) -> Result<PaginatedPosts, String> {
        let limit = (first.unwrap_or(DEFAULT_PAGE_SIZE as i32) as usize).min(MAX_PAGE_SIZE);
        let col = sort_column(&sort_by);

        let filter = if let Some(after_cursor) = after {
            let pc = decode_cursor(after_cursor, &sort_by)?;
            Some(build_keyset_filter(&sort_by, &sort_dir, &pc)?)
        } else {
            None
        };

        let order = match sort_dir {
            SortDirection::Desc => Order::Desc,
            SortDirection::Asc => Order::Asc,
        };

        let rows = PostDao::find_paginated(db, user_id, col, order, filter, (limit + 1) as u64)
            .await
            .map_err(|e| format!("Database error: {}", e))?;

        let has_next_page = rows.len() > limit;
        let has_previous_page = after.is_some();
        let rows: Vec<Model> = rows.into_iter().take(limit).collect();
        let cursors: Vec<String> = rows.iter().map(|p| encode_cursor(&sort_by, p)).collect();

        Ok(PaginatedPosts {
            posts: rows,
            cursors,
            has_previous_page,
            has_next_page,
        })
    }

    pub async fn get_post(
        db: &DatabaseConnection,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Model>, String> {
        PostDao::find_by_id_for_user(db, id, user_id)
            .await
            .map_err(|e| format!("Database error: {}", e))
    }
}

#[cfg(test)]
mod tests {
    use super::super::*;
    use crate::test_helpers::*;

    // ============= pagination tests =============

    #[tokio::test]
    async fn test_default_page_size() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_default").await;

        for i in 0..25 {
            create_test_post(&db, user.id, &format!("Post {}", i), "c", false).await;
        }

        let result = PostRepository::get_posts(
            &db, user.id, None, None, PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        assert_eq!(result.posts.len(), 20);
        assert!(result.has_next_page);
        assert!(!result.has_previous_page);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_first_limits_results() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_first").await;

        for i in 0..5 {
            create_test_post(&db, user.id, &format!("Post {}", i), "c", false).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let result = PostRepository::get_posts(
            &db, user.id, None, Some(2), PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        assert_eq!(result.posts.len(), 2);
        assert!(result.has_next_page);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_after_cursor_returns_next_page() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_after").await;

        for i in 0..4 {
            create_test_post(&db, user.id, &format!("Post {}", i), "c", false).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let page1 = PostRepository::get_posts(
            &db, user.id, None, Some(2), PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        let last_cursor = page1.cursors.last().unwrap();
        let page2 = PostRepository::get_posts(
            &db, user.id, Some(last_cursor.as_str()), Some(2),
            PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        assert_eq!(page2.posts.len(), 2);
        assert!(page2.has_previous_page);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_has_next_page_false_on_last_page() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_lastp").await;

        create_test_post(&db, user.id, "Only Post", "c", false).await;

        let result = PostRepository::get_posts(
            &db, user.id, None, Some(10), PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        assert!(!result.has_next_page);
        assert!(!result.has_previous_page);

        cleanup_user_by_email(&db, &email).await;
    }

    // ============= cursor validation tests =============

    #[tokio::test]
    async fn test_invalid_cursor_returns_error() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_badcur").await;

        let result = PostRepository::get_posts(
            &db, user.id, Some("not-a-valid-cursor"), Some(10),
            PostSortBy::CreatedAt, SortDirection::Desc,
        ).await;

        assert!(result.is_err());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_cursor_sort_mismatch_returns_error() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_mismatch").await;

        for i in 0..3 {
            create_test_post(&db, user.id, &format!("Post {}", i), "c", false).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let page1 = PostRepository::get_posts(
            &db, user.id, None, Some(1), PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        let cursor = page1.cursors.last().unwrap();
        let result = PostRepository::get_posts(
            &db, user.id, Some(cursor.as_str()), Some(1),
            PostSortBy::Title, SortDirection::Desc,
        ).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("sort mismatch"));

        cleanup_user_by_email(&db, &email).await;
    }

    // ============= sort tests =============

    #[tokio::test]
    async fn test_ordered_by_created_at_desc() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_order").await;

        create_test_post(&db, user.id, "First", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Second", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Third", "c", false).await;

        let result = PostRepository::get_posts(
            &db, user.id, None, None, PostSortBy::CreatedAt, SortDirection::Desc,
        ).await.unwrap();

        assert_eq!(result.posts[0].title, "Third");
        assert_eq!(result.posts[1].title, "Second");
        assert_eq!(result.posts[2].title, "First");

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_sort_created_at_asc() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_asc").await;

        create_test_post(&db, user.id, "First", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Second", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Third", "c", false).await;

        let result = PostRepository::get_posts(
            &db, user.id, None, None, PostSortBy::CreatedAt, SortDirection::Asc,
        ).await.unwrap();

        assert_eq!(result.posts[0].title, "First");
        assert_eq!(result.posts[1].title, "Second");
        assert_eq!(result.posts[2].title, "Third");

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_sort_by_title_asc() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_title").await;

        create_test_post(&db, user.id, "Charlie", "c", false).await;
        create_test_post(&db, user.id, "Alpha", "c", false).await;
        create_test_post(&db, user.id, "Bravo", "c", false).await;

        let result = PostRepository::get_posts(
            &db, user.id, None, None, PostSortBy::Title, SortDirection::Asc,
        ).await.unwrap();

        assert_eq!(result.posts[0].title, "Alpha");
        assert_eq!(result.posts[1].title, "Bravo");
        assert_eq!(result.posts[2].title, "Charlie");

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_sort_by_updated_at_desc() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_upd").await;

        let p1 = create_test_post(&db, user.id, "Old", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Middle", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "New", "c", false).await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let mut am: models::posts::ActiveModel = p1.into();
        am.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc());
        am.update(&db).await.unwrap();

        let result = PostRepository::get_posts(
            &db, user.id, None, None, PostSortBy::UpdatedAt, SortDirection::Desc,
        ).await.unwrap();

        assert_eq!(result.posts[0].title, "Old");

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_title_asc_pagination() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "repo_title_pg").await;

        create_test_post(&db, user.id, "Delta", "c", false).await;
        create_test_post(&db, user.id, "Alpha", "c", false).await;
        create_test_post(&db, user.id, "Charlie", "c", false).await;
        create_test_post(&db, user.id, "Bravo", "c", false).await;

        let page1 = PostRepository::get_posts(
            &db, user.id, None, Some(2), PostSortBy::Title, SortDirection::Asc,
        ).await.unwrap();

        assert_eq!(page1.posts[0].title, "Alpha");
        assert_eq!(page1.posts[1].title, "Bravo");
        assert!(page1.has_next_page);

        let cursor = page1.cursors.last().unwrap();
        let page2 = PostRepository::get_posts(
            &db, user.id, Some(cursor.as_str()), Some(2),
            PostSortBy::Title, SortDirection::Asc,
        ).await.unwrap();

        assert_eq!(page2.posts[0].title, "Charlie");
        assert_eq!(page2.posts[1].title, "Delta");
        assert!(!page2.has_next_page);

        cleanup_user_by_email(&db, &email).await;
    }
}
