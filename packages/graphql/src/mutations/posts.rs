use async_graphql::{Context, Object};
use sea_orm::*;
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use crate::types::post::Post as PostType;

#[derive(Default)]
pub struct PostMutation;

trait PostMutations {
    async fn add_post(&self, ctx: &Context<'_>, title: String, body: String) -> Result<PostType, DbErr>;
}

#[Object]
impl PostMutations for PostMutation {
    async fn add_post(&self, ctx: &Context<'_>, title: String, body: String) -> Result<PostType, DbErr> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let post = posts::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(title),
            body: ActiveValue::set(body),
            ..Default::default()
        };

        let res = Posts::insert(post).exec(db).await?;
        
        let p = Posts::find_by_id(res.last_insert_id)
        .one(db)
        .await
        .map(|post| post.unwrap()).unwrap();

        Ok::<PostType, DbErr>(PostType {
            id: res.last_insert_id,
            title: p.title,
            body: p.body,
            published_at: p.published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        })
    }
}
