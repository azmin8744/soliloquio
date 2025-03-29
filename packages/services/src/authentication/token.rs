use crate::claims::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation, errors::Error};
use chrono::{Duration, Utc};
use uuid::Uuid;
use std::{env, fmt};
use models::{prelude::*, *};

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

impl Token {
    pub fn get_user_id(&self) -> Result<Uuid, AuthError> {
        let token_data = self.verify_token()?;
        let user_id = token_data.claims.sub.clone();
        Ok(user_id.parse::<Uuid>().unwrap())
    }

    pub fn verify_token(&self) -> Result<TokenData<Claims>, Error> {
        // a token begins with "Bearer " and then the token
        let token = self.0.split_whitespace().nth(1).unwrap();

        decode::<Claims>(token, &DecodingKey::from_secret(secret().as_ref()), & Validation::default())
    }
}

pub fn generate_token(user: &users::Model) -> String {
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

fn secret() -> String {
    env::var("TOKEN_SECRET").unwrap()
}

fn expiration() -> i64 {
    env::var("TOKEN_EXPIRATION_SECONDS").unwrap().parse::<i64>().unwrap()
}
