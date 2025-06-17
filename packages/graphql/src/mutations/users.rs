use async_graphql::{Context, InputObject, Object, Result, SimpleObject, Union};
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
use services::validation::ValidationError;

// Error Types
#[derive(SimpleObject, Debug)]
pub struct ValidationErrorType {
    message: String,
}

#[derive(SimpleObject, Debug)]
pub struct DbErr {
    message: String,
}

impl From<sea_orm::error::DbErr> for DbErr {
    fn from(e: sea_orm::error::DbErr) -> Self {
        DbErr { message: e.to_string() }
    }
}

#[derive(SimpleObject, Debug)]
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

#[derive(Union)]
pub enum UserMutationResult {
    AuthorizedUser(AuthorizedUser),
    ValidationError(ValidationErrorType),
    DbError(DbErr),
    AuthError(AuthError),
}

// Retain the SignInError struct for backward compatibility and conversions
#[derive(Debug)]
struct SignInError {
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

impl From<ValidationError> for SignInError {
    fn from(e: ValidationError) -> Self {
        SignInError { message: e.to_string() }
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
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<UserMutationResult>;
    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<UserMutationResult>;
    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<UserMutationResult>;
    async fn logout(&self, ctx: &Context<'_>, refresh_token: String) -> Result<bool, AuthError>;
    async fn logout_all_devices(&self, ctx: &Context<'_>, access_token: String) -> Result<bool, AuthError>;
}

#[Object]
impl UserMutations for UserMutation {
    async fn sign_up(&self, ctx: &Context<'_>, input: SignUpInput) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        
        // Create user model for validation
        let mut user = users::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            email: ActiveValue::set(input.email.clone()),
            password: ActiveValue::set(input.password.clone()),
            ..Default::default()
        };
        
        // Validate the user model before proceeding
        use services::validation::ActiveModelValidator;
        if let Err(validation_error) = user.validate() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: validation_error.to_string() 
            }));
        }
        
        // Generate salt and hash the password
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = match argon2.hash_password(&input.password.into_bytes(), &salt) {
            Ok(hash) => hash.to_string(),
            Err(_) => return Ok(UserMutationResult::AuthError(AuthError {
                message: "Failed to hash password".to_string()
            })),
        };
        
        user.password = ActiveValue::set(password_hash.clone());
 
        let res = match user.insert(db).await {
            Ok(user) => user,
            Err(e) => return Ok(UserMutationResult::DbError(DbErr {
                message: e.to_string()
            })),
        };

        // Create refresh token and store in separate table
        let refresh_token = match create_refresh_token(db, res.id, None).await {
            Ok(token) => token,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string()
            })),
        };

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        // Generate access token and return AuthorizedUser
        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: generate_token(&res),
            refresh_token,
        }))
    }

    async fn sign_in(&self, ctx: &Context<'_>, input: SignInInput) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();
        
        // Basic input validation
        if input.email.trim().is_empty() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: "Email cannot be empty".to_string() 
            }));
        }
        
        if input.password.trim().is_empty() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: "Password cannot be empty".to_string() 
            }));
        }
        
        // Find user
        let user = match Users::find()
            .filter(users::Column::Email.contains(input.email))
            .one(db)
            .await 
        {
            Ok(Some(user)) => user,
            Ok(None) => return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: "Email or password incorrect".to_string() 
            })),
            Err(e) => return Ok(UserMutationResult::DbError(DbErr {
                message: e.to_string()
            })),
        };

        // Verify password
        let parsed_hash = match PasswordHash::new(&user.password) {
            Ok(hash) => hash,
            Err(_) => return Ok(UserMutationResult::AuthError(AuthError { 
                message: "Invalid password hash in database".to_string() 
            })),
        };

        if Argon2::default().verify_password(&input.password.into_bytes(), &parsed_hash).is_err() {
            return Ok(UserMutationResult::ValidationError(ValidationErrorType { 
                message: "Email or password incorrect".to_string() 
            }));
        }

        // Create new refresh token for this session
        let refresh_token = match create_refresh_token(db, user.id, None).await {
            Ok(token) => token,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string()
            })),
        };

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;
        
        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: generate_token(&user),
            refresh_token,
        }))
    }

    async fn refresh_access_token(&self, ctx: &Context<'_>, refresh_token: String) -> Result<UserMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        // Validate the refresh token and get the refresh token record
        let refresh_token_record = match validate_refresh_token(db, &refresh_token).await {
            Ok(record) => record,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.message
            })),
        };

        // Get the user associated with this refresh token
        let user = match services::authentication::authenticator::get_user(db, &Token::new(refresh_token.clone())).await {
            Ok(user) => user,
            Err(e) => return Ok(UserMutationResult::AuthError(AuthError {
                message: e.to_string()
            })),
        };
        
        // Ensure the token belongs to the correct user (extra security check)
        if refresh_token_record.user_id != user.id {
            return Ok(UserMutationResult::AuthError(AuthError {
                message: "Refresh token does not belong to the authenticated user".to_string(),
            }));
        }

        // Generate a new access token
        let new_access_token = generate_token(&user);

        // Opportunistic cleanup of expired tokens
        let _ = cleanup_expired_tokens(db).await;

        Ok(UserMutationResult::AuthorizedUser(AuthorizedUser {
            token: new_access_token,
            refresh_token: refresh_token, // Return the same refresh token
        }))
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
