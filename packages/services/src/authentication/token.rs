use crate::claims::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation, errors::Error};
use chrono::{Duration, Utc};
use uuid::Uuid;
use std::{env, fmt};
use models::users::Model as User;

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
    pub fn get_user_id(&self) -> Result<Uuid, AuthError> {
        // If token string begins from "Bearer ", then remove it
        // and get the token
        let token = match self.0.starts_with("Bearer ") {
            true => self.0.split_whitespace().nth(1).unwrap(),
            false => self.0.as_str(),
        };
        let token_data = self.verify(token.to_string())?;
        let user_id = token_data.claims.sub.clone();
        Ok(user_id.parse::<Uuid>().unwrap())
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

fn secret() -> String {
    env::var("TOKEN_SECRET").unwrap()
}

fn expiration() -> i64 {
    env::var("TOKEN_EXPIRATION_SECONDS").unwrap().parse::<i64>().unwrap()
}

fn refresh_token_expiration() -> i64 {
    env::var("REFRESH_TOKEN_EXPIRATION_DAYS").unwrap().parse::<i64>().unwrap()
}