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
use services::authentication::token::{Token, generate_token, generate_refresh_token};

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

pub struct AuthError {
    message: String,
}
impl From<services::AuthenticationError> for AuthError {
    fn from(e: services::AuthenticationError) -> Self {
        AuthError { message: e.to_string() }
    }
}

impl From<sea_orm::error::DbErr> for AuthError {
    fn from(e: sea_orm::error::DbErr) -> Self {
        AuthError { message: e.to_string() }
    }
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

#[derive(Default)]
pub struct UserMutation;

trait UserMutations {
    async fn sign_up(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthorizedUser, SignInError>;
    async fn sign_in(&self, ctx: &Context<'_>, email: String, password: String) -> Result<AuthorizedUser, SignInError>;
    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<AuthorizedUser, AuthError>;
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
        let user_id = Uuid::new_v4();
        let refresh_token = generate_refresh_token(user_id.to_string());
        // User モデルを作ってinsertする
        let user = users::ActiveModel {
            id: ActiveValue::set(user_id),
            email: ActiveValue::set(email),
            password: ActiveValue::set(password_hash),
            refresh_token: ActiveValue::set(Some(refresh_token)),
            ..Default::default()
        };
        let res = user.insert(db).await?;

        // アクセストークンを生成して、AuthorizedUser を返す
        Ok::<AuthorizedUser, SignInError>(AuthorizedUser {
            token: generate_token(&res),
            refresh_token: res.refresh_token.unwrap(),
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

        Argon2::default().verify_password(&password.into_bytes(), &parsed_hash).or(
            Err(SignInError { message: "Password is incorrect".to_string() })
        )?;

        // リフレッシュトークンを生成して保存する
        let refresh_token = generate_refresh_token(user.id.to_string());
        let mut u = user.into_active_model();
        u.refresh_token = Set(Some(refresh_token.clone()));
        let res = u.update(db).await?;
        
        Ok::<AuthorizedUser, SignInError>(AuthorizedUser {
            token: generate_token(&res),
            refresh_token: res.refresh_token.unwrap(),
        })
    }

    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<AuthorizedUser, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let user = services::authentication::authenticator::get_user(db, &Token(refresh_token.clone())).await?;
        
        // Return error if a token saved in the database does not match the token passed in
        if user.refresh_token != Some(refresh_token.clone()) {
            return Err::<AuthorizedUser, AuthError>(AuthError {
                message: "Refresh token is invalid".to_string(),
            });
        }
;
        let refresh_token = generate_refresh_token(user.id.to_string());
        let mut u = user.into_active_model();
        u.refresh_token = Set(Some(refresh_token.clone()));
        let res = u.update(db).await?;

        Ok::<AuthorizedUser, AuthError>(AuthorizedUser {
            token: generate_token(&res),
            refresh_token: res.refresh_token.unwrap(),
        })
    }
}
