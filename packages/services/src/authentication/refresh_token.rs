use sea_orm::*;
use chrono::Utc;
use uuid::Uuid;
use models::refresh_tokens::{self, Entity as RefreshTokens, Model as RefreshToken};
use super::token::{generate_refresh_token, hash_token, AuthError, Token};

/// Create a new refresh token and store it in the database
pub async fn create_refresh_token(
    db: &DatabaseConnection,
    user_id: Uuid,
    device_info: Option<String>,
) -> Result<String, DbErr> {
    let token = generate_refresh_token(user_id.to_string());
    let token_hash = hash_token(&token);
    let expires_at = Token::new(token.clone()).get_expiration().ok().unwrap();
    let refresh_token_model = refresh_tokens::ActiveModel {
        id: ActiveValue::set(Uuid::new_v4()),
        user_id: ActiveValue::set(user_id),
        token_hash: ActiveValue::set(token_hash),
        expires_at: ActiveValue::set(expires_at),
        device_info: ActiveValue::set(device_info),
        created_at: ActiveValue::set(Utc::now().naive_utc()),
        last_used_at: ActiveValue::set(None),
    };

    refresh_token_model.insert(db).await?;
    Ok(token)
}

/// Validate a refresh token and update its last_used_at timestamp
pub async fn validate_refresh_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<RefreshToken, AuthError> {
    let token_hash = hash_token(token);
    let now = Utc::now().naive_utc();

    // Find the refresh token by hash and ensure it's not expired
    let refresh_token = RefreshTokens::find()
        .filter(refresh_tokens::Column::TokenHash.eq(token_hash))
        .filter(refresh_tokens::Column::ExpiresAt.gt(now))
        .one(db)
        .await
        .map_err(|e| AuthError { message: e.to_string() })?
        .ok_or(AuthError {
            message: "Invalid or expired refresh token".to_string(),
        })?;

    // Update last_used_at timestamp
    let mut active_model = refresh_token.clone().into_active_model();
    active_model.last_used_at = ActiveValue::set(Some(now));
    active_model
        .update(db)
        .await
        .map_err(|e| AuthError { message: e.to_string() })?;

    Ok(refresh_token)
}

/// Revoke a specific refresh token
pub async fn revoke_refresh_token(
    db: &DatabaseConnection,
    token: &str,
) -> Result<(), AuthError> {
    let token_hash = hash_token(token);

    RefreshTokens::delete_many()
        .filter(refresh_tokens::Column::TokenHash.eq(token_hash))
        .exec(db)
        .await
        .map_err(|e| AuthError { message: e.to_string() })?;

    Ok(())
}

/// Revoke all refresh tokens for a user (logout from all devices)
pub async fn revoke_all_refresh_tokens(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<(), AuthError> {
    RefreshTokens::delete_many()
        .filter(refresh_tokens::Column::UserId.eq(user_id))
        .exec(db)
        .await
        .map_err(|e| AuthError { message: e.to_string() })?;

    Ok(())
}

/// Clean up expired refresh tokens (opportunistic cleanup)
pub async fn cleanup_expired_tokens(db: &DatabaseConnection) -> Result<u64, DbErr> {
    let now = Utc::now().naive_utc();

    let result = RefreshTokens::delete_many()
        .filter(refresh_tokens::Column::ExpiresAt.lt(now))
        .exec(db)
        .await?;

    Ok(result.rows_affected)
}

/// List all active refresh tokens for a user (for session management)
pub async fn list_user_sessions(
    db: &DatabaseConnection,
    user_id: Uuid,
) -> Result<Vec<RefreshToken>, DbErr> {
    let now = Utc::now().naive_utc();

    RefreshTokens::find()
        .filter(refresh_tokens::Column::UserId.eq(user_id))
        .filter(refresh_tokens::Column::ExpiresAt.gt(now))
        .order_by_desc(refresh_tokens::Column::LastUsedAt)
        .order_by_desc(refresh_tokens::Column::CreatedAt)
        .all(db)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::*;
    use chrono::Duration;

    // ============= Token Generation (sync tests) =============

    #[test]
    fn test_refresh_token_functionality() {
        dotenvy::dotenv().ok();
        let user_id = Uuid::new_v4();

        let token1 = generate_refresh_token(user_id.to_string());
        let token2 = generate_refresh_token(user_id.to_string());

        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());

        let hash1 = hash_token(&token1);
        let hash2 = hash_token(&token2);
        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_multi_device_token_generation() {
        dotenvy::dotenv().ok();
        let user_id = Uuid::new_v4();
        let devices = vec!["iPhone 15", "MacBook Pro", "iPad"];

        let mut generated_tokens = Vec::new();
        let mut generated_hashes = Vec::new();

        for _ in devices.iter() {
            let token = generate_refresh_token(user_id.to_string());
            let token_hash = hash_token(&token);

            assert!(!token.is_empty());
            assert!(!token_hash.is_empty());
            assert!(!generated_tokens.contains(&token));
            assert!(!generated_hashes.contains(&token_hash));

            generated_tokens.push(token);
            generated_hashes.push(token_hash);
        }

        assert_eq!(generated_tokens.len(), devices.len());
    }

    // ============= create_refresh_token =============

    #[tokio::test]
    async fn test_create_inserts_token_in_db() {
        let db = setup_test_db().await;
        let email = format!("test_create_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();

        assert!(!token.is_empty());

        // verify token exists in db
        let token_hash = hash_token(&token);
        let found = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_some());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_create_stores_hash_not_plaintext() {
        let db = setup_test_db().await;
        let email = format!("test_hash_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();

        // db should NOT contain the plaintext token
        let found_by_plaintext = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token))
            .one(&db)
            .await
            .unwrap();
        assert!(found_by_plaintext.is_none());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_create_stores_correct_user_id() {
        let db = setup_test_db().await;
        let email = format!("test_userid_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();
        let token_hash = hash_token(&token);

        let record = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(record.user_id, user.id);

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_create_stores_device_info() {
        let db = setup_test_db().await;
        let email = format!("test_device_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;
        let device = "iPhone 15 - Safari".to_string();

        let token = create_refresh_token(&db, user.id, Some(device.clone()))
            .await
            .unwrap();
        let token_hash = hash_token(&token);

        let record = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        assert_eq!(record.device_info, Some(device));

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_create_sets_expires_at() {
        let db = setup_test_db().await;
        let email = format!("test_expires_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();
        let token_hash = hash_token(&token);

        let record = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap()
            .unwrap();

        // expires_at should be ~7 days from now
        let now = Utc::now().naive_utc();
        let expected_min = now + Duration::days(6);
        let expected_max = now + Duration::days(8);
        assert!(record.expires_at > expected_min);
        assert!(record.expires_at < expected_max);

        cleanup_test_user(&db, user.id).await;
    }

    // ============= validate_refresh_token =============

    #[tokio::test]
    async fn test_validate_valid_token_returns_record() {
        let db = setup_test_db().await;
        let email = format!("test_validate_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();
        let result = validate_refresh_token(&db, &token).await;

        assert!(result.is_ok());
        let record = result.unwrap();
        assert_eq!(record.user_id, user.id);

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_validate_updates_last_used_at() {
        let db = setup_test_db().await;
        let email = format!("test_lastused_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();
        let token_hash = hash_token(&token);

        // check last_used_at is initially None
        let before = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert!(before.last_used_at.is_none());

        // validate and check it's now set
        validate_refresh_token(&db, &token).await.unwrap();

        let after = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap()
            .unwrap();
        assert!(after.last_used_at.is_some());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_validate_expired_token_returns_error() {
        let db = setup_test_db().await;
        let email = format!("test_exp_validate_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        // manually insert an expired token
        let token = generate_refresh_token(user.id.to_string());
        let token_hash = hash_token(&token);
        let expired_at = Utc::now().naive_utc() - Duration::hours(1);

        let expired_model = refresh_tokens::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user.id),
            token_hash: ActiveValue::Set(token_hash),
            expires_at: ActiveValue::Set(expired_at),
            device_info: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
            last_used_at: ActiveValue::Set(None),
        };
        expired_model.insert(&db).await.unwrap();

        let result = validate_refresh_token(&db, &token).await;
        assert!(result.is_err());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_validate_nonexistent_token_returns_error() {
        let db = setup_test_db().await;
        let fake_token = "fake.nonexistent.token";

        let result = validate_refresh_token(&db, fake_token).await;
        assert!(result.is_err());
    }

    // ============= revoke_refresh_token =============

    #[tokio::test]
    async fn test_revoke_deletes_from_db() {
        let db = setup_test_db().await;
        let email = format!("test_revoke_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();
        let token_hash = hash_token(&token);

        revoke_refresh_token(&db, &token).await.unwrap();

        let found = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_none());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_revoke_nonexistent_succeeds() {
        let db = setup_test_db().await;
        let fake_token = "fake.token.to.revoke";

        let result = revoke_refresh_token(&db, fake_token).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_revoke_leaves_other_tokens() {
        let db = setup_test_db().await;
        let email = format!("test_revoke_other_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token1 = create_refresh_token(&db, user.id, Some("Device 1".to_string()))
            .await
            .unwrap();
        let token2 = create_refresh_token(&db, user.id, Some("Device 2".to_string()))
            .await
            .unwrap();
        let token2_hash = hash_token(&token2);

        revoke_refresh_token(&db, &token1).await.unwrap();

        // token2 should still exist
        let found = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token2_hash))
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_some());

        cleanup_test_user(&db, user.id).await;
    }

    // ============= revoke_all_refresh_tokens =============

    #[tokio::test]
    async fn test_revoke_all_deletes_all_user_tokens() {
        let db = setup_test_db().await;
        let email = format!("test_revokeall_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        create_refresh_token(&db, user.id, Some("Device 1".to_string()))
            .await
            .unwrap();
        create_refresh_token(&db, user.id, Some("Device 2".to_string()))
            .await
            .unwrap();
        create_refresh_token(&db, user.id, Some("Device 3".to_string()))
            .await
            .unwrap();

        revoke_all_refresh_tokens(&db, user.id).await.unwrap();

        let count = RefreshTokens::find()
            .filter(refresh_tokens::Column::UserId.eq(user.id))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(count, 0);

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_revoke_all_leaves_other_users() {
        let db = setup_test_db().await;
        let email1 = format!("test_revokeall1_{}@example.com", Uuid::new_v4());
        let email2 = format!("test_revokeall2_{}@example.com", Uuid::new_v4());
        let user1 = create_test_user(&db, &email1, "password").await;
        let user2 = create_test_user(&db, &email2, "password").await;

        create_refresh_token(&db, user1.id, None).await.unwrap();
        create_refresh_token(&db, user2.id, None).await.unwrap();

        revoke_all_refresh_tokens(&db, user1.id).await.unwrap();

        // user2's token should still exist
        let count = RefreshTokens::find()
            .filter(refresh_tokens::Column::UserId.eq(user2.id))
            .count(&db)
            .await
            .unwrap();
        assert_eq!(count, 1);

        cleanup_test_user(&db, user1.id).await;
        cleanup_test_user(&db, user2.id).await;
    }

    // ============= cleanup_expired_tokens =============

    #[tokio::test]
    async fn test_cleanup_removes_expired() {
        let db = setup_test_db().await;
        let email = format!("test_cleanup_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        // insert expired token
        let token = generate_refresh_token(user.id.to_string());
        let token_hash = hash_token(&token);
        let expired_at = Utc::now().naive_utc() - Duration::hours(1);

        let expired_model = refresh_tokens::ActiveModel {
            id: ActiveValue::Set(Uuid::new_v4()),
            user_id: ActiveValue::Set(user.id),
            token_hash: ActiveValue::Set(token_hash.clone()),
            expires_at: ActiveValue::Set(expired_at),
            device_info: ActiveValue::Set(None),
            created_at: ActiveValue::Set(Utc::now().naive_utc()),
            last_used_at: ActiveValue::Set(None),
        };
        expired_model.insert(&db).await.unwrap();

        // run cleanup (don't check count - other tests may run cleanup concurrently)
        cleanup_expired_tokens(&db).await.unwrap();

        // verify our expired token was removed
        let found = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_none());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_cleanup_keeps_valid() {
        let db = setup_test_db().await;
        let email = format!("test_cleanup_valid_{}@example.com", Uuid::new_v4());
        let user = create_test_user(&db, &email, "password").await;

        let token = create_refresh_token(&db, user.id, None).await.unwrap();
        let token_hash = hash_token(&token);

        cleanup_expired_tokens(&db).await.unwrap();

        let found = RefreshTokens::find()
            .filter(refresh_tokens::Column::TokenHash.eq(&token_hash))
            .one(&db)
            .await
            .unwrap();
        assert!(found.is_some());

        cleanup_test_user(&db, user.id).await;
    }

    #[tokio::test]
    async fn test_cleanup_returns_count() {
        let db = setup_test_db().await;

        // just verify the function returns a u64 and doesn't error
        let result = cleanup_expired_tokens(&db).await;
        assert!(result.is_ok());
        // result is a count (could be 0 if no expired tokens exist)
        let _count: u64 = result.unwrap();
    }
}
