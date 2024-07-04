use async_graphql::{Context, Object};
use sea_orm::*;
use models::prelude::*;
use sea_orm::entity::prelude::Uuid;
use crate::queries::Queries;
use crate::types::post::Post as PostType;

trait PostQueries {
    async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<PostType>, DbErr>;
    async fn post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<PostType>, DbErr>;
}

#[Object]
impl PostQueries for Queries {
    async fn posts(&self, ctx: &Context<'_>) -> Result<Vec<PostType>, DbErr> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let res = Posts::find().all(db).await;
        assert_eq!(res.is_ok(), true);
        let posts = res.unwrap();
        let mut vec: Vec<PostType> = Vec::new();
        for post in &posts {
            let p = PostType { 
                id: post.id,
                title: post.title.clone(),
                body: post.body.clone(),
                published_at: post.published_at.clone(),
                created_at: post.created_at,
                updated_at: post.updated_at,
             };
            vec.push(p);
        }
        Ok::<Vec<PostType>, DbErr>(vec)
    }

    async fn post(&self, ctx: &Context<'_>, id: Uuid) -> Result<Option<PostType>, DbErr> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let res = Posts::find_by_id(id).one(db).await;
        assert_eq!(res.is_ok(), true);
        let post = res.unwrap().unwrap();
        let p = PostType { 
            id: post.id,
            title: post.title.clone(),
            body: post.body.clone(),
            published_at: post.published_at.clone(),
            created_at: post.created_at,
            updated_at: post.updated_at,
         };
        Ok::<Option<PostType>, DbErr>(Some(p))
    }
}