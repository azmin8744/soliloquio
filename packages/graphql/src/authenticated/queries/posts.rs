use crate::errors::AuthError;
use crate::types::post::Post as PostType;
use crate::types::sort::{PostSortBy, SortDirection};
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::{Context, Object, Result};
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

impl From<PostSortBy> for repositories::PostSortBy {
    fn from(v: PostSortBy) -> Self {
        match v {
            PostSortBy::CreatedAt => Self::CreatedAt,
            PostSortBy::UpdatedAt => Self::UpdatedAt,
            PostSortBy::Title => Self::Title,
        }
    }
}

impl From<SortDirection> for repositories::SortDirection {
    fn from(v: SortDirection) -> Self {
        match v {
            SortDirection::Asc => Self::Asc,
            SortDirection::Desc => Self::Desc,
        }
    }
}

fn model_to_post_type(p: &models::posts::Model) -> PostType {
    PostType {
        id: p.id,
        title: p.title.clone(),
        markdown_content: p.markdown_content.clone().unwrap_or_default(),
        description: p.description.clone(),
        slug: p.slug.clone(),
        is_published: p.is_published,
        first_published_at: p.first_published_at,
        created_at: p.created_at,
        updated_at: p.updated_at,
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
        search: Option<String>,
    ) -> Result<Connection<String, PostType, EmptyFields, EmptyFields>> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let search = search.as_deref().map(str::trim).filter(|s| !s.is_empty());
        if let Some(q) = search {
            let posts = repositories::PostRepository::search_posts(db, user.id, q)
                .await
                .map_err(|e| async_graphql::Error::new(e))?;
            let mut connection = Connection::new(false, false);
            for post in &posts {
                connection
                    .edges
                    .push(Edge::new(post.id.to_string(), model_to_post_type(post)));
            }
            return Ok(connection);
        }

        let sort_by = sort_by.unwrap_or(PostSortBy::CreatedAt);
        let sort_dir = sort_direction.unwrap_or(SortDirection::Desc);

        let result = repositories::PostRepository::get_posts(
            db,
            user.id,
            after.as_deref(),
            first,
            sort_by.into(),
            sort_dir.into(),
        )
        .await
        .map_err(|e| async_graphql::Error::new(e))?;

        let mut connection = Connection::new(result.has_previous_page, result.has_next_page);
        for (post, cursor) in result.posts.iter().zip(result.cursors.iter()) {
            connection
                .edges
                .push(Edge::new(cursor.clone(), model_to_post_type(post)));
        }

        Ok(connection)
    }

    /// Get a specific post by ID for the authenticated user
    async fn post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<PostType>, AuthError> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let post = repositories::PostRepository::get_post(db, user.id, id)
            .await
            .map_err(|e| AuthError { message: e })?;

        Ok(post.map(|p| model_to_post_type(&p)))
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

}
