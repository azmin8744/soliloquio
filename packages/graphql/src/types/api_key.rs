use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct ApiKeyInfo {
    pub id: Uuid,
    pub label: String,
    pub last_used_at: Option<NaiveDateTime>,
    pub created_at: NaiveDateTime,
}
