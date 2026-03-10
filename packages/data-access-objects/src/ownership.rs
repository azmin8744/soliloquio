use sea_orm::{ColumnTrait, DatabaseConnection, DbErr, EntityTrait, PrimaryKeyTrait, QueryFilter};
use uuid::Uuid;

pub trait OwnedEntity: EntityTrait {
    type UserIdColumn: ColumnTrait;
    fn user_id_column() -> Self::UserIdColumn;
}

pub async fn verify_ownership<E>(
    db: &DatabaseConnection,
    id: Uuid,
    user_id: Uuid,
) -> Result<Option<E::Model>, DbErr>
where
    E: OwnedEntity,
    <E as EntityTrait>::PrimaryKey: PrimaryKeyTrait<ValueType = Uuid>,
{
    E::find_by_id(id)
        .filter(E::user_id_column().eq(user_id))
        .one(db)
        .await
}
