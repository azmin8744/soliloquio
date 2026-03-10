use crate::ownership::OwnedEntity;
use models::assets::Column;

impl OwnedEntity for models::assets::Entity {
    type UserIdColumn = Column;
    fn user_id_column() -> Self::UserIdColumn {
        Column::UserId
    }
}
