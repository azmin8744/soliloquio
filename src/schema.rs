// src/schema.rs

use async_graphql::{Context, Object};
use models::{prelude::*, *};
use entity::prelude::Uuid;
use sea_orm::*;

pub(crate) struct QueryRoot;
pub(crate) struct MutationRoot;

#[Object]
impl QueryRoot {
    async fn hello(&self) -> String {
        "Hello GraphQL".to_owned()
    }
}

#[Object]
impl MutationRoot {
    async fn add_post(&self, ctx: &Context<'_>, title: String, body: String) -> Result<posts::Model, DbErr> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let post = posts::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(title),
            body: ActiveValue::set(body),
            ..Default::default()
        };

        let res = Posts::insert(post).exec(db).await?;

        Posts::find_by_id(res.last_insert_id)
        .one(db)
        .await
        .map(|post| post.unwrap())
    }
}
