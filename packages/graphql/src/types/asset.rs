use async_graphql::{Object, SimpleObject};
use chrono::NaiveDateTime;
use services::assets::StorageDriver;
use std::sync::Arc;
use uuid::Uuid;

pub struct Asset {
    pub id: Uuid,
    pub original_filename: String,
    pub mime_type: String,
    pub size_bytes: i64,
    pub created_at: NaiveDateTime,
}

impl From<models::assets::Model> for Asset {
    fn from(m: models::assets::Model) -> Self {
        Asset {
            id: m.id,
            original_filename: m.original_filename,
            mime_type: m.mime_type,
            size_bytes: m.size_bytes,
            created_at: m.created_at,
        }
    }
}

#[derive(SimpleObject)]
pub struct AssetUrls {
    pub thumbnail: String,
    pub small: String,
    pub medium: String,
    pub large: String,
    pub original: String,
}

#[derive(SimpleObject)]
pub struct DeletedAsset {
    pub id: Uuid,
}

#[Object]
impl Asset {
    async fn id(&self) -> Uuid { self.id }
    async fn original_filename(&self) -> &str { &self.original_filename }
    async fn mime_type(&self) -> &str { &self.mime_type }
    async fn size_bytes(&self) -> i64 { self.size_bytes }
    async fn created_at(&self) -> NaiveDateTime { self.created_at }

    async fn urls(&self, ctx: &async_graphql::Context<'_>) -> AssetUrls {
        let driver = ctx
            .data::<Arc<StorageDriver>>()
            .expect("StorageDriver not in context");
        let base = self.id.to_string();
        AssetUrls {
            thumbnail: driver.url(&format!("{base}/thumbnail.webp")),
            small: driver.url(&format!("{base}/small.webp")),
            medium: driver.url(&format!("{base}/medium.webp")),
            large: driver.url(&format!("{base}/large.webp")),
            original: driver.url(&format!("{base}/original.webp")),
        }
    }
}
