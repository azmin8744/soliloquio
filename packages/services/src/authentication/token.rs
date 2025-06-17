use crate::claims::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation, errors::Error};
use chrono::{Duration, Utc, DateTime, NaiveDateTime};
use uuid::Uuid;
use std::{env, fmt};
use models::users::Model as User;
use sha2::{Sha256, Digest};
use base64::{engine::general_purpose, Engine};

pub struct Token(pub String);
pub struct AuthError {
    pub message: String,
}

impl From<jsonwebtoken::errors::Error> for AuthError {
    fn from(e: jsonwebtoken::errors::Error) -> Self {
        AuthError { message: e.to_string() }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

trait TokenTrait {
    fn verify(&self, token: String) -> Result<TokenData<Claims>, Error> {
        decode::<Claims>(&token, &DecodingKey::from_secret(secret().as_ref()), &Validation::default())
    }
}
impl TokenTrait for Token {}

impl Token {
    /// Create a new Token instance from a token string
    pub fn new(token_string: String) -> Self {
        Token(token_string)
    }

    /// Get the raw token string, removing "Bearer " prefix if present
    pub fn get_token_string(&self) -> &str {
        match self.0.starts_with("Bearer ") {
            true => self.0.split_whitespace().nth(1).unwrap_or(&self.0),
            false => &self.0,
        }
    }

    /// Get the token claims
    pub fn get_claims(&self) -> Result<Claims, AuthError> {
        let token_data = self.verify(self.get_token_string().to_string())?;
        Ok(token_data.claims)
    }

    /// Get user ID from token
    pub fn get_user_id(&self) -> Result<Uuid, AuthError> {
        let claims = self.get_claims()?;
        let user_id = claims.sub.parse::<Uuid>()
            .map_err(|_| AuthError { message: "Invalid user ID in token".to_string() })?;
        Ok(user_id)
    }

    /// Get token expiration timestamp
    pub fn get_expiration(&self) -> Result<NaiveDateTime, AuthError> {
        let claims = self.get_claims()?;
        let expiration = DateTime::from_timestamp(claims.exp, 0)
            .ok_or(AuthError { message: "Invalid expiration timestamp".to_string() })?
            .naive_utc();
        Ok(expiration)
    }

    /// Get token issued-at timestamp
    pub fn get_issued_at(&self) -> Result<NaiveDateTime, AuthError> {
        let claims = self.get_claims()?;
        let issued_at = DateTime::from_timestamp(claims.iat, 0)
            .ok_or(AuthError { message: "Invalid issued-at timestamp".to_string() })?
            .naive_utc();
        Ok(issued_at)
    }

    /// Get token JTI (JWT ID)
    pub fn get_jti(&self) -> Result<String, AuthError> {
        let claims = self.get_claims()?;
        Ok(claims.jti)
    }

    /// Check if token is expired
    pub fn is_expired(&self) -> Result<bool, AuthError> {
        let expiration = self.get_expiration()?;
        Ok(Utc::now().naive_utc() > expiration)
    }

    /// Check if token is valid (not expired and properly signed)
    pub fn is_valid(&self) -> bool {
        match self.get_claims() {
            Ok(_) => !self.is_expired().unwrap_or(true),
            Err(_) => false,
        }
    }
}

pub fn generate_token(user: &User) -> String {
    let expiration = Utc::now().checked_add_signed(Duration::seconds(expiration())).unwrap();
    let host_name = env::var("HOST_NAME").unwrap();
    let claims = Claims {
        iss: host_name,
        sub: user.id.clone().to_string(),
        exp: expiration.timestamp(),
        iat: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret().as_ref())).unwrap()
}

pub fn generate_refresh_token(user_id: String) -> String {
    let expiration = Utc::now().checked_add_signed(Duration::days(refresh_token_expiration())).unwrap();
    let host_name = env::var("HOST_NAME").unwrap();
    let claims = Claims {
        iss: host_name,
        sub: user_id,
        exp: expiration.timestamp(),
        iat: Utc::now().timestamp(),
        jti: Uuid::new_v4().to_string(),
    };

    encode(&Header::default(), &claims, &EncodingKey::from_secret(secret().as_ref())).unwrap()
}

/// Hash a token string using SHA256 for secure storage
pub fn hash_token(token: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(token.as_bytes());
    let result = hasher.finalize();
    general_purpose::STANDARD_NO_PAD.encode(result)
}

fn secret() -> String {
    env::var("TOKEN_SECRET").unwrap()
}

fn expiration() -> i64 {
    env::var("TOKEN_EXPIRATION_SECONDS").unwrap().parse::<i64>().unwrap()
}

fn refresh_token_expiration() -> i64 {
    env::var("REFRESH_TOKEN_EXPIRATION_DAYS").unwrap().parse::<i64>().unwrap()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_parsing_and_validation() {
        // Test Bearer token parsing
        let bearer_token = "Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test";
        let token = Token::new(bearer_token.to_string());
        let parsed = token.get_token_string();
        assert_eq!(parsed, "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test");
        
        // Test direct token parsing
        let direct_token = "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.test";
        let token2 = Token::new(direct_token.to_string());
        let parsed2 = token2.get_token_string();
        assert_eq!(parsed2, direct_token);
        
        println!("✅ Token parsing works correctly");
    }

    #[test]
    fn test_token_hashing() {
        // Test token hashing functionality  
        let original_token = "test_token_123";
        let hash1 = hash_token(original_token);
        let hash2 = hash_token(original_token);
        
        // Hashes should be identical for the same input
        assert_eq!(hash1, hash2);
        
        // Different tokens should produce different hashes
        let different_token = "different_token_456";
        let hash3 = hash_token(different_token);
        assert_ne!(hash1, hash3);
        
        // Hash should not be empty and should be base64 encoded
        assert!(!hash1.is_empty());
        assert!(hash1.len() > 40); // SHA256 hash should be 43 chars when base64 encoded
        
        println!("✅ Token hashing works correctly");
    }
}