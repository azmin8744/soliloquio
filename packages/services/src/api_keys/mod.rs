use models::api_keys::{self, Entity as ApiKeys};
use sea_orm::*;
use sha2::{Digest, Sha256};
use uuid::Uuid;

fn hash_key(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(raw.as_bytes());
    format!("{:x}", hasher.finalize())
}

/// Returns `(raw_key, key_hash)`. Raw key shown once to user.
pub fn generate() -> (String, String) {
    let raw = format!("slq_{}", Uuid::new_v4().simple());
    let hash = hash_key(&raw);
    (raw, hash)
}

pub async fn create(
    db: &DatabaseConnection,
    user_id: Uuid,
    label: String,
    key_hash: String,
) -> Result<api_keys::Model, DbErr> {
    api_keys::ActiveModel {
        user_id: ActiveValue::set(user_id),
        key_hash: ActiveValue::set(key_hash),
        label: ActiveValue::set(label),
        ..Default::default()
    }
    .insert(db)
    .await
}

/// Returns `Some(user_id)` if key is valid, updates last_used_at fire-and-forget.
pub async fn validate(db: &DatabaseConnection, raw_key: &str) -> Option<Uuid> {
    let hash = hash_key(raw_key);
    let record = ApiKeys::find()
        .filter(api_keys::Column::KeyHash.eq(&hash))
        .one(db)
        .await
        .ok()??;

    let user_id = record.user_id;
    // Fire-and-forget: update last_used_at
    let mut am = record.into_active_model();
    am.last_used_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
    let _ = am.update(db).await;

    Some(user_id)
}

pub async fn revoke(db: &DatabaseConnection, key_id: Uuid, user_id: Uuid) -> Result<(), DbErr> {
    let deleted = ApiKeys::delete_many()
        .filter(api_keys::Column::Id.eq(key_id))
        .filter(api_keys::Column::UserId.eq(user_id))
        .exec(db)
        .await?;
    if deleted.rows_affected == 0 {
        return Err(DbErr::RecordNotFound("API key not found".to_string()));
    }
    Ok(())
}

pub async fn list(db: &DatabaseConnection, user_id: Uuid) -> Result<Vec<api_keys::Model>, DbErr> {
    ApiKeys::find()
        .filter(api_keys::Column::UserId.eq(user_id))
        .order_by_desc(api_keys::Column::CreatedAt)
        .all(db)
        .await
}
