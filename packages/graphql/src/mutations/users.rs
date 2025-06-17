use async_graphql::{Context, InputObject, Object};
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
use services::authentication::refresh_token::{
    create_refresh_token, validate_refresh_token, revoke_refresh_token, revoke_all_refresh_tokens, cleanup_expired_tokens
};

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

#[derive(InputObject)]
struct SignUpInput {
    email: String,
    password: String,
}

#[derive(InputObject)]
struct SignInInput {
    email: String,
    password: String,
}

#[derive(Default)]
pub struct UserMutation;

trait UserMutations {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<AuthorizedUser, SignInError>;
    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<AuthorizedUser, SignInError>;
    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<AuthorizedUser, AuthError>;
    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError>;
    async fn logout_all_devices(&self, ctx: &Context<'_>, access_token: String) -> Result<bool, AuthError>;
}

#[Object]
impl UserMutations for UserMutation {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<AuthorizedUser, SignInError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        // ソルトをランダムに生成する
        let salt = SaltString::generate(&mut OsRng);
        // パスワードをscryptでハッシュ化する
        let argon2 = Argon2::default();
        let password_hash = argon2.hash_password(&input.password.into_bytes(), &salt)?.to_string();
        let user_id = Uuid::new_v4();
        
        // User モデルを作ってinsertする
        let user = users::ActiveModel {
            id: ActiveValue::set(user_id),
            email: ActiveValue::set(input.email),
            password: ActiveValue::set(password_hash),
            ..Default::default()
        };
        let res = user.insert(db).await?;

        // Create refresh token and store in separate table
        let refresh_token = create_refresh_token(db, res.id, None).await
            .map_err(|e| SignInError { message: e.to_string() })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        // アクセストークンを生成して、AuthorizedUser を返す
        Ok::<AuthorizedUser, SignInError>(AuthorizedUser {
            token: generate_token(&res),
            refresh_token,
        })
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<AuthorizedUser, SignInError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        let user = Users::find()
        .filter(users::Column::Email.contains(input.email))
        .one(db)
        .await?
        .ok_or(SignInError { message: "User not found".to_string() })?;

        let parsed_hash = PasswordHash::new(&user.password)?;

        Argon2::default().verify_password(&input.password.into_bytes(), &parsed_hash).or(
            Err(SignInError { message: "Password is incorrect".to_string() })
        )?;

        // Create new refresh token for this session
        let refresh_token = create_refresh_token(db, user.id, None).await
            .map_err(|e| SignInError { message: e.to_string() })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;
        
        Ok::<AuthorizedUser, SignInError>(AuthorizedUser {
            token: generate_token(&user),
            refresh_token,
        })
    }

    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<AuthorizedUser, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Validate the refresh token and get the refresh token record
        let refresh_token_record = validate_refresh_token(db, &refresh_token).await
            .map_err(|e| AuthError { message: e.message })?;

        // Get the user associated with this refresh token
        let user = services::authentication::authenticator::get_user(db, &Token::new(refresh_token.clone())).await?;
        
        // Ensure the token belongs to the correct user (extra security check)
        if refresh_token_record.user_id != user.id {
            return Err::<AuthorizedUser, AuthError>(AuthError {
                message: "Refresh token does not belong to the authenticated user".to_string(),
            });
        }

        // Generate a new access token
        let new_access_token = generate_token(&user);

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok::<AuthorizedUser, AuthError>(AuthorizedUser {
            token: new_access_token,
            refresh_token: refresh_token, // Return the same refresh token
        })
    }

    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Revoke the specific refresh token
        revoke_refresh_token(db, &refresh_token).await
            .map_err(|e| AuthError { message: e.message })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(true)
    }

    async fn logout_all_devices(&self, ctx: &Context<'_>, access_token: String) -> Result<bool, AuthError> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Get user from access token
        let token = Token::new(access_token);
        let user = services::authentication::authenticator::get_user(db, &token).await?;

        // Revoke all refresh tokens for this user
        revoke_all_refresh_tokens(db, user.id).await
            .map_err(|e| AuthError { message: e.message })?;

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(true)
    }
}
