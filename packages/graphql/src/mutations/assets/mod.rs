use crate::errors::{AuthError, DbError};
use crate::types::asset::DeletedAsset;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::{Context, Object, Result, Union};
use sea_orm::entity::prelude::Uuid;
use services::assets::StorageDriver;
use std::sync::Arc;

#[derive(Union)]
pub enum AssetMutationResult {
    DeletedAsset(DeletedAsset),
    DbError(DbError),
    AuthError(AuthError),
}

#[derive(Default)]
pub struct AssetMutation;

impl RequiresAuth for AssetMutation {}

#[Object]
impl AssetMutation {
    async fn delete_asset(&self, ctx: &Context<'_>, id: Uuid) -> Result<AssetMutationResult> {
        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(u) => u,
            Err(e) => return Ok(AssetMutationResult::AuthError(AuthError { message: e.to_string() })),
        };

        let db = ctx.data::<sea_orm::DatabaseConnection>().unwrap();
        let driver = ctx.data::<Arc<StorageDriver>>().unwrap();

        // Ownership check
        let asset = repositories::AssetRepository::get(db, user.id, id).await;
        let asset = match asset {
            Ok(Some(a)) => a,
            Ok(None) => return Ok(AssetMutationResult::AuthError(AuthError { message: "Asset not found".to_string() })),
            Err(e) => return Ok(AssetMutationResult::DbError(DbError { message: e })),
        };

        // Delete files from storage
        if let Err(e) = driver.delete_dir(&asset.id.to_string()).await {
            tracing::warn!("storage delete_dir failed: {e}");
        }

        match repositories::AssetRepository::delete(db, user.id, id).await {
            Ok(_) => Ok(AssetMutationResult::DeletedAsset(DeletedAsset { id })),
            Err(e) => Ok(AssetMutationResult::DbError(DbError { message: e })),
        }
    }
}
