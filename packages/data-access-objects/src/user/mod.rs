use models::users::{ActiveModel, Column, Entity, Model};
use models::prelude::Users;
use sea_orm::*;

pub struct UserDao;

impl UserDao {
    pub async fn count(db: &DatabaseConnection) -> Result<u64, DbErr> {
        Users::find().count(db).await
    }

    pub async fn insert(db: &DatabaseConnection, model: ActiveModel) -> Result<Model, DbErr> {
        let res = Users::insert(model).exec(db).await?;
        Users::find_by_id(res.last_insert_id)
            .one(db)
            .await?
            .ok_or(DbErr::Custom("Inserted user not found".to_string()))
    }

    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        Users::find().filter(Column::Email.eq(email)).one(db).await
    }

    pub async fn find_by_email_contains(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        Users::find()
            .filter(Column::Email.contains(email))
            .one(db)
            .await
    }

    pub async fn update(db: &DatabaseConnection, model: ActiveModel) -> Result<Model, DbErr> {
        Entity::update(model).exec(db).await
    }
}
