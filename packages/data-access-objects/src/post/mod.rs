use models::posts::{ActiveModel, Column, Entity, Model};
use models::prelude::Posts;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

pub struct PostDao;

impl PostDao {
    pub async fn find_by_id(
        db: &DatabaseConnection,
        id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Posts::find_by_id(id).one(db).await
    }

    pub async fn find_by_id_for_user(
        db: &DatabaseConnection,
        id: Uuid,
        user_id: Uuid,
    ) -> Result<Option<Model>, DbErr> {
        Posts::find_by_id(id)
            .filter(Column::UserId.eq(user_id))
            .one(db)
            .await
    }

    pub async fn find_paginated(
        db: &DatabaseConnection,
        user_id: Uuid,
        sort_col: Column,
        order: Order,
        filter: Option<Condition>,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr> {
        let mut q = Posts::find().filter(Column::UserId.eq(user_id));

        if let Some(cond) = filter {
            q = q.filter(cond);
        }

        q.order_by(sort_col, order.clone())
            .order_by(Column::Id, order)
            .limit(limit)
            .all(db)
            .await
    }

    pub async fn insert(
        db: &DatabaseConnection,
        model: ActiveModel,
    ) -> Result<Model, DbErr> {
        let res = Posts::insert(model).exec(db).await?;
        Posts::find_by_id(res.last_insert_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Inserted post not found".to_string()))
    }

    pub async fn update(
        db: &DatabaseConnection,
        model: ActiveModel,
    ) -> Result<Model, DbErr> {
        Entity::update(model).exec(db).await
    }

    pub async fn delete(
        db: &DatabaseConnection,
        model: ActiveModel,
    ) -> Result<DeleteResult, DbErr> {
        Entity::delete(model).exec(db).await
    }
}
