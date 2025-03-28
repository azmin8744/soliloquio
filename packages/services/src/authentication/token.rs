use crate::claims::Claims;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, TokenData, Validation, errors::Error};
use chrono::{Duration, Utc};
use uuid::Uuid;
use std::env;
use models::{prelude::*, *};

pub struct Token(pub String);

impl Token {
    pub fn verify_token(&self, token: &str) -> Result<TokenData<Claims>, Error> {
        // a token begins with "Bearer " and then the token
        let token = token.split_whitespace().nth(1).unwrap();

        decode::<Claims>(token, &DecodingKey::from_secret(secret().as_ref()), &Validation::default())
    }
}

pub fn generate_token(user: &users::Model) -> String {
    let expiration = Utc::now().checked_add_signed(Duration::seconds(expiration())).unwrap();
    let host_name = env::var("HOST_NAME").unwrap();
    let claims = Claims {
        iss: host_name,
        sub: "AccessToken".to_string(),
        aud: user.id.clone().to_string(),
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
// #[cfg(test)]
// mod tests {
//     use super::*;

//     #[test]
//     fn test_verify_token() {
//         let
//     }
// }


// impl Token {
//     pub fn new(secret: String, expiration: i64) -> Self {
//         Self {
//             secret,
//             expiration,
//         }
//     }

//     pub fn generate_token(&self, user: &User) -> String {
//         let expiration = Utc::now().checked_add_signed(Duration::seconds(self.expiration)).unwrap();
//         let claims = Claims {
//             sub: user.id.to_string(),
//             exp: expiration.timestamp(),
//         };

//         encode(&Header::default(), &claims, &EncodingKey::from_secret(self.secret.as_ref())).unwrap()
//     }

//     pub fn decode_token(&self, token: &str) -> Result<Claims, Error> {
//         decode::<Claims>(token, &DecodingKey::from_secret(self.secret.as_ref()), &Validation::default())
//     }
// }