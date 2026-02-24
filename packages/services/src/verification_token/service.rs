use chrono::Utc;
use models::verification_tokens::{self, Entity as VerificationTokens};
use sea_orm::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

pub use models::sea_orm_active_enums::TokenKind;

#[derive(Debug)]
pub struct TokenError {
    pub message: String,
}

impl TokenError {
    fn new(msg: &str) -> Self {
        TokenError { message: msg.to_string() }
    }
}

impl From<DbErr> for TokenError {
    fn from(e: DbErr) -> Self {
        TokenError { message: e.to_string() }
    }
}

fn hash_token(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

pub async fn create_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    kind: TokenKind,
    expires_in_seconds: i64,
) -> Result<String, TokenError> {
    let raw_token = Uuid::new_v4().to_string();
    let token_hash = hash_token(&raw_token);
    let expires_at = Utc::now().naive_utc() + chrono::Duration::seconds(expires_in_seconds);

    let record = verification_tokens::ActiveModel {
        user_id: ActiveValue::set(user_id),
        token_hash: ActiveValue::set(token_hash),
        kind: ActiveValue::set(kind),
        expires_at: ActiveValue::set(expires_at),
        ..Default::default()
    };

    record.insert(db).await?;
    Ok(raw_token)
}

pub async fn validate_token(
    db: &DatabaseConnection,
    raw_token: &str,
    kind: TokenKind,
) -> Result<models::verification_tokens::Model, TokenError> {
    let token_hash = hash_token(raw_token);

    let record = VerificationTokens::find()
        .filter(verification_tokens::Column::TokenHash.eq(&token_hash))
        .filter(verification_tokens::Column::Kind.eq(kind))
        .one(db)
        .await?
        .ok_or_else(|| TokenError::new("Invalid or expired token"))?;

    if record.used_at.is_some() {
        return Err(TokenError::new("Token already used"));
    }

    if record.expires_at < Utc::now().naive_utc() {
        return Err(TokenError::new("Token expired"));
    }

    let mut active = record.clone().into_active_model();
    active.used_at = ActiveValue::set(Some(Utc::now().naive_utc()));
    active.update(db).await?;

    Ok(record)
}

pub async fn cleanup_expired(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let result = VerificationTokens::delete_many()
        .filter(verification_tokens::Column::ExpiresAt.lt(Utc::now().naive_utc()))
        .exec(db)
        .await?;
    Ok(result.rows_affected)
}
