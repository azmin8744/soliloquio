use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct User {
    pub id: Uuid,
    pub email: String,
    pub created_at: Option<NaiveDateTime>,
    pub updated_at: Option<NaiveDateTime>,
}
