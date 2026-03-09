use models::assets::{self, ActiveModel, Column, Entity, Model};
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

const DEFAULT_PAGE_SIZE: u64 = 20;
const MAX_PAGE_SIZE: u64 = 100;

pub struct AssetRepository;

impl AssetRepository {
    pub async fn create(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
        original_filename: String,
        mime_type: String,
        size_bytes: i64,
    ) -> Result<Model, DbErr> {
        let am = ActiveModel {
            id: ActiveValue::Set(id),
            user_id: ActiveValue::Set(user_id),
            original_filename: ActiveValue::Set(original_filename),
            mime_type: ActiveValue::Set(mime_type),
            size_bytes: ActiveValue::Set(size_bytes),
            created_at: ActiveValue::Set(chrono::Utc::now().naive_utc()),
        };
        Entity::insert(am)
            .exec_with_returning(db)
            .await
    }

    pub async fn list(
        db: &DatabaseConnection,
        user_id: Uuid,
        after_id: Option<Uuid>,
        after_created_at: Option<chrono::NaiveDateTime>,
        limit: Option<u64>,
    ) -> Result<Vec<Model>, String> {
        let limit = limit.unwrap_or(DEFAULT_PAGE_SIZE).min(MAX_PAGE_SIZE);

        let mut q = Entity::find()
            .filter(Column::UserId.eq(user_id))
            .order_by_desc(Column::CreatedAt)
            .order_by_desc(Column::Id);

        if let (Some(at), Some(aid)) = (after_created_at, after_id) {
            q = q.filter(
                Condition::any()
                    .add(Column::CreatedAt.lt(at))
                    .add(
                        Condition::all()
                            .add(Column::CreatedAt.eq(at))
                            .add(Column::Id.lt(aid)),
                    ),
            );
        }

        q.limit(limit + 1)
            .all(db)
            .await
            .map_err(|e| format!("Database error: {e}"))
    }

    pub async fn get(
        db: &DatabaseConnection,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<Option<Model>, String> {
        Entity::find_by_id(id)
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
            .map_err(|e| format!("Database error: {e}"))
    }

    pub async fn delete(
        db: &DatabaseConnection,
        user_id: Uuid,
        id: Uuid,
    ) -> Result<bool, String> {
        let result = Entity::delete_many()
            .filter(Column::Id.eq(id))
            .filter(Column::UserId.eq(user_id))
            .exec(db)
            .await
            .map_err(|e| format!("Database error: {e}"))?;
        Ok(result.rows_affected > 0)
    }
}

// Re-export for cursor encoding
pub use assets::Model as AssetModel;
pub const ASSET_DEFAULT_PAGE_SIZE: u64 = DEFAULT_PAGE_SIZE;
