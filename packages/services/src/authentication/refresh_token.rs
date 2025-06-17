use sea_orm::*;
use chrono::{Duration, Utc};
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
    use uuid::Uuid;

    #[test]
    fn test_refresh_token_functionality() {
        // Test refresh token specific functionality
        let user_id = Uuid::new_v4();
        
        // Test token generation
        let token1 = generate_refresh_token(user_id.to_string());
        let token2 = generate_refresh_token(user_id.to_string());
        
        // Each refresh token should be unique
        assert_ne!(token1, token2);
        assert!(!token1.is_empty());
        assert!(!token2.is_empty());
        
        // Test that hashes are different for different tokens
        let hash1 = hash_token(&token1);
        let hash2 = hash_token(&token2);
        assert_ne!(hash1, hash2);
        
        println!("✅ Refresh token generation works correctly");
    }

    #[test]
    fn test_multi_device_token_generation() {
        // Test that multiple tokens are generated uniquely for different devices
        let user_id = Uuid::new_v4();
        
        // Simulate user signing in from multiple devices
        let devices = vec![
            "iPhone 15 - Safari",
            "MacBook Pro - Chrome", 
            "iPad - Safari",
        ];
        
        println!("🔐 Multi-device refresh token system test");
        println!("User ID: {}", user_id);
        
        let mut generated_tokens = Vec::new();
        let mut generated_hashes = Vec::new();
        
        // Generate tokens for each device
        for (i, device) in devices.iter().enumerate() {
            let token = generate_refresh_token(user_id.to_string());
            let token_hash = hash_token(&token);
            
            println!("Device {}: {} - Token hash: {}", i + 1, device, &token_hash[..20]);
            
            // Each token should be unique and not empty
            assert!(!token.is_empty());
            assert!(!token_hash.is_empty());
            assert!(!generated_tokens.contains(&token));
            assert!(!generated_hashes.contains(&token_hash));
            
            generated_tokens.push(token);
            generated_hashes.push(token_hash);
        }
        
        // Verify all tokens are unique
        assert_eq!(generated_tokens.len(), devices.len());
        assert_eq!(generated_hashes.len(), devices.len());
        
        println!("✅ Multi-device token generation works correctly");
    }
}
