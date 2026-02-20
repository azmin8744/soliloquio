use crate::errors::AuthError;
use crate::types::post::Post as PostType;
use crate::types::sort::{PostSortBy, SortDirection};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::{Context, Object, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use models::prelude::*;
use sea_orm::entity::prelude::Uuid;
use sea_orm::sea_query::{Expr, SimpleExpr};
use sea_orm::*;
use serde::{Deserialize, Serialize};

const DEFAULT_PAGE_SIZE: usize = 20;
const MAX_PAGE_SIZE: usize = 100;

#[derive(Serialize, Deserialize)]
struct PostCursor {
    s: String, // sort column tag
    v: String, // sort value serialized
    i: Uuid,   // tiebreaker
}

fn sort_tag(sort_by: &PostSortBy) -> &'static str {
    match sort_by {
        PostSortBy::CreatedAt => "c",
        PostSortBy::UpdatedAt => "u",
        PostSortBy::Title => "t",
    }
}

fn encode_cursor(sort_by: &PostSortBy, post: &models::posts::Model) -> String {
    let v = match sort_by {
        PostSortBy::CreatedAt => post.created_at.and_utc().to_rfc3339(),
        PostSortBy::UpdatedAt => post.updated_at.and_utc().to_rfc3339(),
        PostSortBy::Title => post.title.clone(),
    };
    let cursor = PostCursor {
        s: sort_tag(sort_by).to_string(),
        v,
        i: post.id,
    };
    let json = serde_json::to_string(&cursor).unwrap();
    URL_SAFE_NO_PAD.encode(json.as_bytes())
}

fn decode_cursor(
    cursor: &str,
    expected: &PostSortBy,
) -> Result<PostCursor, async_graphql::Error> {
    let bytes = URL_SAFE_NO_PAD
        .decode(cursor)
        .map_err(|_| async_graphql::Error::new("Invalid cursor"))?;
    let json =
        String::from_utf8(bytes).map_err(|_| async_graphql::Error::new("Invalid cursor"))?;
    let pc: PostCursor =
        serde_json::from_str(&json).map_err(|_| async_graphql::Error::new("Invalid cursor"))?;
    if pc.s != sort_tag(expected) {
        return Err(async_graphql::Error::new(
            "Cursor sort mismatch: cursor was created with a different sort order",
        ));
    }
    Ok(pc)
}

fn sort_column(sort_by: &PostSortBy) -> models::posts::Column {
    match sort_by {
        PostSortBy::CreatedAt => models::posts::Column::CreatedAt,
        PostSortBy::UpdatedAt => models::posts::Column::UpdatedAt,
        PostSortBy::Title => models::posts::Column::Title,
    }
}

#[derive(Default)]
pub struct PostQueries;

impl RequiresAuth for PostQueries {}

#[Object]
impl PostQueries {
    /// Get paginated posts for the authenticated user
    async fn posts(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
        sort_by: Option<PostSortBy>,
        sort_direction: Option<SortDirection>,
    ) -> Result<Connection<String, PostType, EmptyFields, EmptyFields>> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let sort_by = sort_by.unwrap_or(PostSortBy::CreatedAt);
        let sort_dir = sort_direction.unwrap_or(SortDirection::Desc);
        let limit = (first.unwrap_or(DEFAULT_PAGE_SIZE as i32) as usize).min(MAX_PAGE_SIZE);
        let col = sort_column(&sort_by);

        let mut q = Posts::find().filter(models::posts::Column::UserId.eq(user.id));

        if let Some(ref after_cursor) = after {
            let pc = decode_cursor(after_cursor, &sort_by)?;

            // Parse cursor value back to the right sea_orm Value
            let cursor_val: sea_orm::Value = match sort_by {
                PostSortBy::CreatedAt | PostSortBy::UpdatedAt => {
                    let dt = chrono::DateTime::parse_from_rfc3339(&pc.v)
                        .map_err(|_| async_graphql::Error::new("Invalid cursor"))?;
                    dt.naive_utc().into()
                }
                PostSortBy::Title => pc.v.clone().into(),
            };

            // Keyset filter: DESC → (col < val) OR (col = val AND id < id)
            //                 ASC  → (col > val) OR (col = val AND id > id)
            let (col_cmp, id_cmp): (
                fn(models::posts::Column, sea_orm::Value) -> SimpleExpr,
                fn(models::posts::Column, Uuid) -> SimpleExpr,
            ) = match sort_dir {
                SortDirection::Desc => (
                    |c, v| Expr::col((models::posts::Entity, c)).lt(v),
                    |c, id| Expr::col((models::posts::Entity, c)).lt(id),
                ),
                SortDirection::Asc => (
                    |c, v| Expr::col((models::posts::Entity, c)).gt(v),
                    |c, id| Expr::col((models::posts::Entity, c)).gt(id),
                ),
            };

            q = q.filter(
                Condition::any()
                    .add(col_cmp(col, cursor_val.clone()))
                    .add(
                        Condition::all()
                            .add(Expr::col((models::posts::Entity, col)).eq(cursor_val))
                            .add(id_cmp(models::posts::Column::Id, pc.i)),
                    ),
            );
        }

        let order = match sort_dir {
            SortDirection::Desc => Order::Desc,
            SortDirection::Asc => Order::Asc,
        };

        let rows = q
            .order_by(col, order.clone())
            .order_by(models::posts::Column::Id, order)
            .limit((limit + 1) as u64)
            .all(db)
            .await
            .map_err(|e| async_graphql::Error::new(format!("Database error: {}", e)))?;

        let has_next_page = rows.len() > limit;
        let has_previous_page = after.is_some();
        let rows = &rows[..rows.len().min(limit)];

        let mut connection = Connection::new(has_previous_page, has_next_page);
        for post in rows {
            connection.edges.push(Edge::new(
                encode_cursor(&sort_by, post),
                PostType {
                    id: post.id,
                    title: post.title.clone(),
                    markdown_content: post.markdown_content.clone().unwrap_or_default(),
                    is_published: post.is_published,
                    first_published_at: post.first_published_at,
                    created_at: post.created_at,
                    updated_at: post.updated_at,
                },
            ));
        }

        Ok(connection)
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
                message: format!("Database error: {}", e),
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

    const POSTS_QUERY: &str = r#"query($first: Int, $after: String, $sortBy: PostSortBy, $sortDirection: SortDirection) {
        posts(first: $first, after: $after, sortBy: $sortBy, sortDirection: $sortDirection) {
            edges { node { id title } cursor }
            pageInfo { hasNextPage hasPreviousPage endCursor }
        }
    }"#;

    // ============= posts() connection tests =============

    #[tokio::test]
    async fn test_posts_authenticated_returns_user_posts_only() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_auth");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "Post 1", "content 1", false).await;
        create_test_post(&db, user.id, "Post 2", "content 2", true).await;

        let res = schema
            .execute(Request::new(POSTS_QUERY).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);

        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();
        assert_eq!(edges.len(), 2);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_unauthenticated_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());

        let res = schema.execute(Request::new(POSTS_QUERY)).await;
        assert!(!res.errors.is_empty());
    }

    #[tokio::test]
    async fn test_posts_empty_returns_empty_edges() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_empty");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let res = schema
            .execute(Request::new(POSTS_QUERY).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();

        let edges = data["posts"]["edges"].as_array().unwrap();
        assert!(edges.is_empty());
        assert_eq!(data["posts"]["pageInfo"]["hasNextPage"], false);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_ordered_by_created_at_desc() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_order");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "First", "content", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Second", "content", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Third", "content", false).await;

        let res = schema
            .execute(Request::new(POSTS_QUERY).data(Token::new(token)))
            .await;
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges[0]["node"]["title"], "Third");
        assert_eq!(edges[1]["node"]["title"], "Second");
        assert_eq!(edges[2]["node"]["title"], "First");

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

        let res = schema
            .execute(Request::new(POSTS_QUERY).data(Token::new(token1)))
            .await;
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges.len(), 1);
        assert_eq!(edges[0]["node"]["title"], "User1 Post");

        cleanup_test_user_by_email(&db, &email1).await;
        cleanup_test_user_by_email(&db, &email2).await;
    }

    // ============= pagination tests =============

    #[tokio::test]
    async fn test_posts_first_limits_results() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_first");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        for i in 0..5 {
            create_test_post(&db, user.id, &format!("Post {}", i), "content", false).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 2}),
                    )),
            )
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges.len(), 2);
        assert_eq!(data["posts"]["pageInfo"]["hasNextPage"], true);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_after_cursor_returns_next_page() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_after");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        for i in 0..4 {
            create_test_post(&db, user.id, &format!("Post {}", i), "content", false).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // First page
        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token.clone()))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 2}),
                    )),
            )
            .await;
        let data = res.data.into_json().unwrap();
        let end_cursor = data["posts"]["pageInfo"]["endCursor"]
            .as_str()
            .unwrap()
            .to_string();

        // Second page using cursor
        let res2 = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 2, "after": end_cursor}),
                    )),
            )
            .await;
        assert!(res2.errors.is_empty(), "Errors: {:?}", res2.errors);
        let data2 = res2.data.into_json().unwrap();
        let edges2 = data2["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges2.len(), 2);
        assert_eq!(data2["posts"]["pageInfo"]["hasPreviousPage"], true);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_has_next_page_false_on_last_page() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_lastp");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "Only Post", "content", false).await;

        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 10}),
                    )),
            )
            .await;
        let data = res.data.into_json().unwrap();

        assert_eq!(data["posts"]["pageInfo"]["hasNextPage"], false);
        assert_eq!(data["posts"]["pageInfo"]["hasPreviousPage"], false);

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_invalid_cursor_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_badcur");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 10, "after": "not-a-valid-cursor"}),
                    )),
            )
            .await;
        assert!(!res.errors.is_empty());

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_default_page_size() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("posts_default");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        // Create 25 posts (more than default 20)
        for i in 0..25 {
            create_test_post(&db, user.id, &format!("Post {}", i), "content", false).await;
        }

        // No first arg — should default to 20
        let res = schema
            .execute(Request::new(POSTS_QUERY).data(Token::new(token)))
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges.len(), 20);
        assert_eq!(data["posts"]["pageInfo"]["hasNextPage"], true);

        cleanup_test_user_by_email(&db, &email).await;
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

        let post = create_test_post(&db, user2.id, "User2 Post", "content", false).await;

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

    // ============= sort tests =============

    #[tokio::test]
    async fn test_posts_sort_by_title_asc() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("sort_title");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "Charlie", "c", false).await;
        create_test_post(&db, user.id, "Alpha", "a", false).await;
        create_test_post(&db, user.id, "Bravo", "b", false).await;

        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"sortBy": "TITLE", "sortDirection": "ASC"}),
                    )),
            )
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges[0]["node"]["title"], "Alpha");
        assert_eq!(edges[1]["node"]["title"], "Bravo");
        assert_eq!(edges[2]["node"]["title"], "Charlie");

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_sort_by_updated_at_desc() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("sort_upd");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        let p1 = create_test_post(&db, user.id, "Old", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _p2 = create_test_post(&db, user.id, "Middle", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        let _p3 = create_test_post(&db, user.id, "New", "c", false).await;

        // Touch p1 to make its updated_at newest
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        use sea_orm::ActiveModelTrait;
        let mut am: models::posts::ActiveModel = p1.into();
        am.updated_at = sea_orm::ActiveValue::Set(chrono::Utc::now().naive_utc());
        am.update(&db).await.unwrap();

        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"sortBy": "UPDATED_AT", "sortDirection": "DESC"}),
                    )),
            )
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        // p1 was touched last, so it should be first
        assert_eq!(edges[0]["node"]["title"], "Old");

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_sort_created_at_asc_reversal() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("sort_asc");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "First", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Second", "c", false).await;
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        create_test_post(&db, user.id, "Third", "c", false).await;

        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"sortBy": "CREATED_AT", "sortDirection": "ASC"}),
                    )),
            )
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();

        assert_eq!(edges[0]["node"]["title"], "First");
        assert_eq!(edges[1]["node"]["title"], "Second");
        assert_eq!(edges[2]["node"]["title"], "Third");

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_cursor_sort_mismatch_returns_error() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("sort_mismatch");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        for i in 0..3 {
            create_test_post(&db, user.id, &format!("Post {}", i), "c", false).await;
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
        }

        // Get cursor with CREATED_AT sort
        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token.clone()))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 1, "sortBy": "CREATED_AT"}),
                    )),
            )
            .await;
        let data = res.data.into_json().unwrap();
        let end_cursor = data["posts"]["pageInfo"]["endCursor"]
            .as_str()
            .unwrap()
            .to_string();

        // Use that cursor with TITLE sort — should error
        let res2 = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 1, "after": end_cursor, "sortBy": "TITLE"}),
                    )),
            )
            .await;
        assert!(!res2.errors.is_empty());
        assert!(
            res2.errors[0].message.contains("sort mismatch"),
            "Expected sort mismatch error, got: {}",
            res2.errors[0].message
        );

        cleanup_test_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_posts_title_asc_pagination() {
        let db = setup_test_db().await;
        let schema = create_test_schema(db.clone());
        let email = generate_unique_email("sort_title_pg");
        let user = create_test_user_with_password(&db, &email, &valid_password()).await;
        let token = create_access_token(&user);

        create_test_post(&db, user.id, "Delta", "c", false).await;
        create_test_post(&db, user.id, "Alpha", "c", false).await;
        create_test_post(&db, user.id, "Charlie", "c", false).await;
        create_test_post(&db, user.id, "Bravo", "c", false).await;

        // Page 1
        let res = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token.clone()))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 2, "sortBy": "TITLE", "sortDirection": "ASC"}),
                    )),
            )
            .await;
        assert!(res.errors.is_empty(), "Errors: {:?}", res.errors);
        let data = res.data.into_json().unwrap();
        let edges = data["posts"]["edges"].as_array().unwrap();
        assert_eq!(edges[0]["node"]["title"], "Alpha");
        assert_eq!(edges[1]["node"]["title"], "Bravo");
        assert_eq!(data["posts"]["pageInfo"]["hasNextPage"], true);

        let end_cursor = data["posts"]["pageInfo"]["endCursor"]
            .as_str()
            .unwrap()
            .to_string();

        // Page 2
        let res2 = schema
            .execute(
                Request::new(POSTS_QUERY)
                    .data(Token::new(token))
                    .variables(async_graphql::Variables::from_json(
                        serde_json::json!({"first": 2, "after": end_cursor, "sortBy": "TITLE", "sortDirection": "ASC"}),
                    )),
            )
            .await;
        assert!(res2.errors.is_empty(), "Errors: {:?}", res2.errors);
        let data2 = res2.data.into_json().unwrap();
        let edges2 = data2["posts"]["edges"].as_array().unwrap();
        assert_eq!(edges2[0]["node"]["title"], "Charlie");
        assert_eq!(edges2[1]["node"]["title"], "Delta");
        assert_eq!(data2["posts"]["pageInfo"]["hasNextPage"], false);

        cleanup_test_user_by_email(&db, &email).await;
    }
}
