use async_graphql::{EmptySubscription, Schema};
use chrono::Utc;
use models::users;
use sea_orm::*;
use uuid::Uuid;

use crate::mutations::Mutations;
use crate::queries::Queries;
use crate::utilities::MarkdownCache;

const DATABASE_URL: &str = "postgres://postgres:password@localhost:5432/soliloquio";

pub type TestSchema = Schema<Queries, Mutations, EmptySubscription>;

pub async fn setup_test_db() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    Database::connect(DATABASE_URL)
        .await
        .expect("Failed to connect to test database")
}

pub fn create_test_schema(db: DatabaseConnection) -> TestSchema {
    let markdown_cache = MarkdownCache::new();
    Schema::build(Queries::default(), Mutations::default(), EmptySubscription)
        .data(db)
        .data(markdown_cache)
        .finish()
}

pub async fn create_test_user_with_password(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
) -> users::Model {
    use argon2::{
        password_hash::{rand_core::OsRng, PasswordHasher, SaltString},
        Argon2,
    };

    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let password_hash = argon2
        .hash_password(password.as_bytes(), &salt)
        .unwrap()
        .to_string();

    let user = users::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        email: ActiveValue::Set(email.to_string()),
        password: ActiveValue::Set(password_hash),
        created_at: ActiveValue::Set(Some(Utc::now().naive_utc())),
        updated_at: ActiveValue::Set(None),
    };

    user.insert(db).await.expect("Failed to create test user")
}

pub async fn cleanup_test_user(db: &DatabaseConnection, user_id: Uuid) {
    users::Entity::delete_by_id(user_id).exec(db).await.ok();
}

pub async fn cleanup_test_user_by_email(db: &DatabaseConnection, email: &str) {
    users::Entity::delete_many()
        .filter(users::Column::Email.eq(email))
        .exec(db)
        .await
        .ok();
}

pub fn generate_unique_email(prefix: &str) -> String {
    format!("{}_{}_@example.com", prefix, Uuid::new_v4())
}

pub fn valid_password() -> String {
    "SecureP@ssw0rd123!".to_string()
}
