use crate::types::asset::Asset;
use crate::utilities::requires_auth::RequiresAuth;
use async_graphql::connection::{Connection, Edge, EmptyFields};
use async_graphql::{Context, Object, Result};
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use repositories::ASSET_DEFAULT_PAGE_SIZE;
use sea_orm::*;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct AssetCursor {
    id: Uuid,
    created_at: String,
}

fn encode_cursor(m: &models::assets::Model) -> String {
    let c = AssetCursor {
        id: m.id,
        created_at: m.created_at.and_utc().to_rfc3339(),
    };
    URL_SAFE_NO_PAD.encode(serde_json::to_string(&c).unwrap())
}

fn decode_cursor(s: &str) -> Option<AssetCursor> {
    let bytes = URL_SAFE_NO_PAD.decode(s).ok()?;
    serde_json::from_slice(&bytes).ok()
}

#[derive(Default)]
pub struct AssetQueries;

impl RequiresAuth for AssetQueries {}

#[Object]
impl AssetQueries {
    async fn assets(
        &self,
        ctx: &Context<'_>,
        after: Option<String>,
        first: Option<i32>,
    ) -> Result<Connection<String, Asset, EmptyFields, EmptyFields>> {
        let user = self.require_authenticate_as_user(ctx).await?;
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let limit = first
            .map(|n| (n as u64).min(100))
            .unwrap_or(ASSET_DEFAULT_PAGE_SIZE);

        let (after_id, after_created_at) = if let Some(ref cursor) = after {
            if let Some(c) = decode_cursor(cursor) {
                let dt = chrono::DateTime::parse_from_rfc3339(&c.created_at)
                    .ok()
                    .map(|d| d.naive_utc());
                (Some(c.id), dt)
            } else {
                (None, None)
            }
        } else {
            (None, None)
        };

        let rows = repositories::AssetRepository::list(
            db,
            user.id,
            after_id,
            after_created_at,
            Some(limit + 1),
        )
        .await
        .map_err(async_graphql::Error::new)?;

        let has_next_page = rows.len() as u64 > limit;
        let has_previous_page = after.is_some();
        let rows: Vec<_> = rows.into_iter().take(limit as usize).collect();

        let mut connection = Connection::new(has_previous_page, has_next_page);
        for row in &rows {
            connection
                .edges
                .push(Edge::new(encode_cursor(row), Asset::from(row.clone())));
        }
        Ok(connection)
    }
}
