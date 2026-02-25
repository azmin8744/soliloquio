use chrono::Utc;
use data_access_objects::UserDao;
use models::users::{ActiveModel, Model};
use sea_orm::entity::prelude::Uuid;
use sea_orm::*;
use services::validation::ActiveModelValidator;

pub struct UserRepository;

impl UserRepository {
    pub async fn count(db: &DatabaseConnection) -> Result<u64, DbErr> {
        UserDao::count(db).await
    }

    pub async fn create(
        db: &DatabaseConnection,
        id: Uuid,
        email: String,
        password_hash: String,
    ) -> Result<Model, String> {
        let model = ActiveModel {
            id: ActiveValue::set(id),
            email: ActiveValue::set(email),
            password: ActiveValue::set(password_hash),
            ..Default::default()
        };
        if let Err(e) = model.validate() {
            return Err(e.to_string());
        }
        UserDao::insert(db, model).await.map_err(|e| e.to_string())
    }

    pub async fn find_by_email(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        UserDao::find_by_email(db, email).await
    }

    pub async fn find_by_email_for_login(
        db: &DatabaseConnection,
        email: &str,
    ) -> Result<Option<Model>, DbErr> {
        UserDao::find_by_email_contains(db, email).await
    }

    pub async fn update_password(
        db: &DatabaseConnection,
        user_id: Uuid,
        new_hash: String,
    ) -> Result<Model, String> {
        let model = ActiveModel {
            id: ActiveValue::set(user_id),
            password: ActiveValue::set(new_hash),
            updated_at: ActiveValue::set(Some(chrono::Utc::now().naive_utc())),
            ..Default::default()
        };
        if let Err(e) = model.validate() {
            return Err(e.to_string());
        }
        UserDao::update(db, model).await.map_err(|e| e.to_string())
    }

    pub async fn update_email(
        db: &DatabaseConnection,
        user: Model,
        new_email: String,
        reset_verified: bool,
    ) -> Result<Model, String> {
        let mut model = user.into_active_model();
        model.email = ActiveValue::set(new_email);
        model.updated_at = ActiveValue::set(Some(Utc::now().naive_utc()));
        if reset_verified {
            model.email_verified_at = ActiveValue::set(None);
        }
        if let Err(e) = model.validate() {
            return Err(e.to_string());
        }
        UserDao::update(db, model).await.map_err(|e| e.to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::UserRepository;
    use crate::test_helpers::*;
    use uuid::Uuid;

    const FAKE_HASH: &str = "$argon2id$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$ZmFrZWhhc2g";

    #[tokio::test]
    async fn test_create_stores_fields() {
        let db = setup_test_db().await;
        let id = Uuid::new_v4();
        let email = format!("user_create_{}@example.com", Uuid::new_v4());

        let user = UserRepository::create(&db, id, email.clone(), FAKE_HASH.to_string())
            .await
            .unwrap();

        assert_eq!(user.id, id);
        assert_eq!(user.email, email);
        assert_eq!(user.password, FAKE_HASH);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_count_increments_after_create() {
        let db = setup_test_db().await;
        let email = format!("user_count_{}@example.com", Uuid::new_v4());

        let before = UserRepository::count(&db).await.unwrap();
        UserRepository::create(&db, Uuid::new_v4(), email.clone(), FAKE_HASH.to_string())
            .await
            .unwrap();
        let after = UserRepository::count(&db).await.unwrap();

        assert!(after >= before + 1);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_find_by_email_finds_user() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "find_exact").await;

        let found = UserRepository::find_by_email(&db, &email).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_find_by_email_returns_none() {
        let db = setup_test_db().await;

        let found = UserRepository::find_by_email(
            &db,
            &format!("noone_{}@example.com", Uuid::new_v4()),
        )
        .await
        .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_find_by_email_no_partial_match() {
        let db = setup_test_db().await;
        let (_, email) = create_test_user(&db, "find_no_partial").await;
        let prefix = email.split('@').next().unwrap();

        let found = UserRepository::find_by_email(&db, prefix).await.unwrap();

        assert!(found.is_none());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_find_by_email_for_login_matches_substring() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "login_sub").await;
        let prefix = email.split('@').next().unwrap().to_string();

        let found = UserRepository::find_by_email_for_login(&db, &prefix)
            .await
            .unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().id, user.id);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_find_by_email_for_login_returns_none() {
        let db = setup_test_db().await;

        let found = UserRepository::find_by_email_for_login(
            &db,
            &format!("ghost_{}@example.com", Uuid::new_v4()),
        )
        .await
        .unwrap();

        assert!(found.is_none());
    }

    #[tokio::test]
    async fn test_update_password_sets_hash() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "upd_pw").await;

        let new_hash = "$argon2id$v=19$m=4096,t=3,p=1$c29tZXNhbHQ$bmV3aGFzaA";
        let updated = UserRepository::update_password(&db, user.id, new_hash.to_string())
            .await
            .unwrap();

        assert_eq!(updated.password, new_hash);

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_password_sets_updated_at() {
        let db = setup_test_db().await;
        let (user, email) = create_test_user(&db, "upd_pw_ts").await;

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated = UserRepository::update_password(&db, user.id, FAKE_HASH.to_string())
            .await
            .unwrap();

        assert!(updated.updated_at.is_some());

        cleanup_user_by_email(&db, &email).await;
    }

    #[tokio::test]
    async fn test_update_email_changes_email() {
        let db = setup_test_db().await;
        let (user, _old_email) = create_test_user(&db, "upd_email").await;
        let new_email = format!("new_{}@example.com", Uuid::new_v4());

        let updated = UserRepository::update_email(&db, user, new_email.clone(), false)
            .await
            .unwrap();

        assert_eq!(updated.email, new_email);

        cleanup_user_by_email(&db, &new_email).await;
    }

    #[tokio::test]
    async fn test_update_email_sets_updated_at() {
        let db = setup_test_db().await;
        let (user, _old_email) = create_test_user(&db, "upd_email_ts").await;
        let new_email = format!("new_ts_{}@example.com", Uuid::new_v4());

        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        let updated = UserRepository::update_email(&db, user, new_email.clone(), false)
            .await
            .unwrap();

        assert!(updated.updated_at.is_some());

        cleanup_user_by_email(&db, &new_email).await;
    }

    #[tokio::test]
    async fn test_update_email_clears_verified_when_reset_true() {
        let db = setup_test_db().await;
        let (user, _old_email) = create_test_user(&db, "upd_clear_vr").await;
        let new_email = format!("cleared_{}@example.com", Uuid::new_v4());

        let updated = UserRepository::update_email(&db, user, new_email.clone(), true)
            .await
            .unwrap();

        assert!(updated.email_verified_at.is_none());

        cleanup_user_by_email(&db, &new_email).await;
    }

    #[tokio::test]
    async fn test_update_email_keeps_verified_when_reset_false() {
        use models::users;
        use sea_orm::{ActiveValue, EntityTrait, IntoActiveModel};

        let db = setup_test_db().await;
        let (user, _old_email) = create_test_user(&db, "upd_keep_vr").await;

        // set email_verified_at in the DB first
        let mut am = user.clone().into_active_model();
        am.email_verified_at = ActiveValue::Set(Some(chrono::Utc::now().naive_utc()));
        let user = users::Entity::update(am).exec(&db).await.unwrap();

        let new_email = format!("kept_{}@example.com", Uuid::new_v4());
        let updated = UserRepository::update_email(&db, user, new_email.clone(), false)
            .await
            .unwrap();

        assert!(updated.email_verified_at.is_some());

        cleanup_user_by_email(&db, &new_email).await;
    }
}
