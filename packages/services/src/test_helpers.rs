use chrono::Utc;
use models::users;
use sea_orm::*;
use uuid::Uuid;

const DATABASE_URL: &str = "postgres://postgres:password@localhost:5432/soliloquio";

pub async fn setup_test_db() -> DatabaseConnection {
    dotenvy::dotenv().ok();
    Database::connect(DATABASE_URL)
        .await
        .expect("Failed to connect to test database")
}

pub async fn create_test_user(
    db: &DatabaseConnection,
    email: &str,
    password: &str,
) -> users::Model {
    let user = users::ActiveModel {
        id: ActiveValue::Set(Uuid::new_v4()),
        email: ActiveValue::Set(email.to_string()),
        password: ActiveValue::Set(password.to_string()),
        created_at: ActiveValue::Set(Some(Utc::now().naive_utc())),
        updated_at: ActiveValue::Set(None),
    };

    user.insert(db).await.expect("Failed to create test user")
}

pub async fn cleanup_test_user(db: &DatabaseConnection, user_id: Uuid) {
    users::Entity::delete_by_id(user_id).exec(db).await.ok();
}

pub fn create_test_token(user: &users::Model) -> crate::authentication::token::Token {
    let token_string = crate::authentication::token::generate_token(user);
    crate::authentication::token::Token::new(token_string)
}

pub fn create_expired_token(user: &users::Model) -> crate::authentication::token::Token {
    use crate::authentication::claims::Claims;
    use chrono::Duration;
    use jsonwebtoken::{encode, EncodingKey, Header};

    let expiration = Utc::now().checked_sub_signed(Duration::hours(1)).unwrap();
    let secret = std::env::var("TOKEN_SECRET").unwrap_or_else(|_| "secret".to_string());

    let claims = Claims {
        iss: "localhost".to_string(),
        sub: user.id.to_string(),
        exp: expiration.timestamp(),
        iat: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    let token_string = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    crate::authentication::token::Token::new(token_string)
}

pub fn create_malformed_token() -> crate::authentication::token::Token {
    crate::authentication::token::Token::new("not.a.valid.jwt.token".to_string())
}

pub fn create_invalid_signature_token(user: &users::Model) -> crate::authentication::token::Token {
    use crate::authentication::claims::Claims;
    use chrono::Duration;
    use jsonwebtoken::{encode, EncodingKey, Header};

    let expiration = Utc::now().checked_add_signed(Duration::hours(1)).unwrap();

    let claims = Claims {
        iss: "localhost".to_string(),
        sub: user.id.to_string(),
        exp: expiration.timestamp(),
        iat: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    let token_string = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret("wrong_secret".as_ref()),
    )
    .unwrap();

    crate::authentication::token::Token::new(token_string)
}

pub fn create_token_for_nonexistent_user() -> crate::authentication::token::Token {
    use crate::authentication::claims::Claims;
    use chrono::Duration;
    use jsonwebtoken::{encode, EncodingKey, Header};

    let expiration = Utc::now().checked_add_signed(Duration::hours(1)).unwrap();
    let secret = std::env::var("TOKEN_SECRET").unwrap_or_else(|_| "secret".to_string());

    let claims = Claims {
        iss: "localhost".to_string(),
        sub: Uuid::new_v4().to_string(), // random user id that doesn't exist
        exp: expiration.timestamp(),
        iat: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    let token_string = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    crate::authentication::token::Token::new(token_string)
}
