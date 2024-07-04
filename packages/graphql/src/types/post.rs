use async_graphql::SimpleObject;
use chrono::NaiveDateTime;
use uuid::Uuid;

#[derive(SimpleObject)]
pub struct Post {
    pub id: Uuid,
    pub title: String,
    pub body: String,
    pub published_at: Option<NaiveDateTime>,
    pub created_at:  Option<NaiveDateTime>,
    pub updated_at:  Option<NaiveDateTime>,
}
