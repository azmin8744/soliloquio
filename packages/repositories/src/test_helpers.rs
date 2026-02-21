use models::posts;
use sea_orm::*;
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@localhost:5432/soliloquio";

pub async fn setup_test_db() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    Database::connect(DATABASE_URL)
        .await
        .expect("Failed to connect to test database")
}

pub async fn create_test_user(db: &DatabaseConnection, prefix: &str) -> (models::users::Model, String) {
    use models::users;

    let email = format!("{}_{}_@example.com", prefix, Uuid::new_v4());
    let user = users::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        email: ActiveValue::Set(email.clone()),
        password: ActiveValue::Set("hashed".to_string()),
        created_at: ActiveValue::Set(Some(chrono::Utc::now().naive_utc())),
        updated_at: ActiveValue::Set(None),
    };

    let model = user.insert(db).await.expect("Failed to create test user");
    (model, email)
}

pub async fn create_test_post(
    db: &DatabaseConnection,
    user_id: Uuid,
    title: &str,
    content: &str,
    is_published: bool,
) -> posts::Model {
    let first_published_at = if is_published {
        Some(chrono::Utc::now().naive_utc())
    } else {
        None
    };

    let post = posts::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        title: ActiveValue::Set(title.to_string()),
        markdown_content: ActiveValue::Set(Some(content.to_string())),
        user_id: ActiveValue::Set(user_id),
        is_published: ActiveValue::Set(is_published),
        first_published_at: ActiveValue::Set(first_published_at),
        created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        updated_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
    };

    post.insert(db).await.expect("Failed to create test post")
}

pub async fn cleanup_user_by_email(db: &DatabaseConnection, email: &str) {
    use models::users;
    users::Entity::delete_many()
        .filter(users::Column::Email.eq(email))
        .exec(db)
        .await
        .ok();
}
