use crate::ownership::{verify_ownership, OwnedEntity};
use chrono::NaiveDateTime;
use models::posts::{ActiveModel, Column, Entity, Model};
use models::prelude::Posts;
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;

impl OwnedEntity for Entity {
    type UserIdColumn = Column;
    fn user_id_column() -> Self::UserIdColumn {
        Column::UserId
    }
}

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
        verify_ownership::<Entity>(db, id, user_id).await
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

    pub async fn find_paginated_published(
        db: &DatabaseConnection,
        user_id: Uuid,
        sort_col: Column,
        order: Order,
        filter: Option<Condition>,
        limit: u64,
    ) -> Result<Vec<Model>, DbErr> {
        let mut q = Posts::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsPublished.eq(true));
        if let Some(cond) = filter {
            q = q.filter(cond);
        }
        q.order_by(sort_col, order.clone())
            .order_by(Column::Id, order)
            .limit(limit)
            .all(db)
            .await
    }

    pub async fn find_prev_published(
        db: &DatabaseConnection,
        user_id: Uuid,
        before: NaiveDateTime,
    ) -> Result<Option<Model>, DbErr> {
        Posts::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsPublished.eq(true))
            .filter(Column::FirstPublishedAt.lt(before))
            .order_by_desc(Column::FirstPublishedAt)
            .one(db)
            .await
    }

    pub async fn find_next_published(
        db: &DatabaseConnection,
        user_id: Uuid,
        after: NaiveDateTime,
    ) -> Result<Option<Model>, DbErr> {
        Posts::find()
            .filter(Column::UserId.eq(user_id))
            .filter(Column::IsPublished.eq(true))
            .filter(Column::FirstPublishedAt.gt(after))
            .order_by_asc(Column::FirstPublishedAt)
            .one(db)
            .await
    }

    pub async fn search_bm25(
        db: &DatabaseConnection,
        user_id: Uuid,
        q: &str,
    ) -> Result<Vec<Model>, DbErr> {
        use sea_orm::{DbBackend, Statement};
        let stmt = Statement::from_sql_and_values(
            DbBackend::Postgres,
            "SELECT id, title, markdown_content, description, slug, user_id,
                    is_published, first_published_at, created_at, updated_at
             FROM posts
             WHERE (title ||| $1 or markdown_content ||| $1 or description ||| $1) AND user_id = $2
             ORDER BY paradedb.score(id) DESC
             LIMIT 50",
            [q.into(), user_id.into()],
        );
        Posts::find().from_raw_sql(stmt).all(db).await
    }
}
