use async_graphql::{Context, Object};
use std::fmt;
use sea_orm::*;
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use argon2::{
    password_hash::{
        rand_core::OsRng,
        PasswordHash, PasswordHasher, PasswordVerifier, SaltString
    },
    Argon2
};
use crate::types::authorized_user::AuthorizedUser;
use services::authentication::token::{Token, generate_token};
use base64::{engine::general_purpose::URL_SAFE, Engine as _};

#[derive(Debug)]
struct SignInError{
    pub message: String,
}

impl SignInError {
    pub fn to_string(&self) -> String {
        self.message.clone()
    }
}

impl fmt::Display for SignInError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.to_string().as_str())
    }
}

impl From<sea_orm::error::DbErr> for SignInError {
    fn from(e: sea_orm::error::DbErr) -> Self {
        SignInError { message: e.to_string() }
    }
}

impl From<argon2::password_hash::Error> for SignInError {
    fn from(e: argon2::password_hash::Error) -> Self {
        SignInError { message: e.to_string() }
    }
}

#[derive(Default)]
pub struct UserMutation;

trait UserMutations {
    async fn sign_up(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthorizedUser, SignInError>;
    async fn sign_in(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthorizedUser, SignInError>;
}

#[Object]
impl UserMutations for UserMutation {
    async fn sign_up(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthorizedUser, SignInError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        // ソルトをランダムに生成する
        let salt = SaltString::generate(&mut OsRng);
        // パスワードをscryptでハッシュ化する
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(&password.into_bytes(), &salt)?.to_string();
        
        // リフレッシュトークンを生成する
        // UUID を base64 エンコードしてリフレッシュトークンとする
        let refresh_token = URL_SAFE.encode(Uuid::new_v4().to_string());
        // User モデルを作ってinsertする
        let user = users::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            email: ActiveValue::set(email),
            password: ActiveValue::set(password_hash),
            refresh_token: ActiveValue::set(Some(refresh_token)),
            ..Default::default()
        };

        let res = Users::insert(user).exec(db).await?;

        let user = Users::find_by_id(res.last_insert_id)
        .one(db)
        .await?
        .unwrap();
        
        // JWT トークンを生成する
        let token = generate_token(&user);
        // AuthorizedUser を返す
        Ok::<AuthorizedUser, SignInError>(AuthorizedUser {
            token: token,
            refresh_token: user.refresh_token.unwrap(),
        })
    }

    async fn sign_in(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthorizedUser, SignInError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let user = Users::find()
        .filter(users::Column::Email.contains(email))
        .one(db)
        .await?
        .ok_or(SignInError { message: "User not found".to_string() })?;

        let parsed_hash = PasswordHash::new(&user.password)?;

        let user = match Argon2::default().verify_password(&password.into_bytes(), &parsed_hash) {
            Ok(_) => AuthorizedUser {
                token: generate_token(&user),
                refresh_token: user.refresh_token.unwrap(),
            },
            Err(_) => return Err(SignInError { message: "Password is incorrect".to_string() }),
        };
        
        Ok::<AuthorizedUser, SignInError>(user)
    }
}
