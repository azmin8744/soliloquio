use async_graphql::{Context, Object, SimpleObject, Union, Result};
use sea_orm::*;
use std::fmt;
use std::error::Error;
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use crate::types::post::Post as PostType;
use crate::utilities::requires_auth::RequiresAuth;
use services::authentication::token::Token;
use services::authentication::authenticator::*;

#[derive(SimpleObject, Debug)]
pub struct DbErr {
    message: String,
}

impl From<sea_orm::error::DbErr> for DbErr {
    fn from(e: sea_orm::error::DbErr) -> Self {
        DbErr { message: e.to_string() }
    }
}

impl fmt::Display for DbErr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

#[derive(SimpleObject, Debug)]
pub struct AuthError {
    message: String,
}
impl From<crate::utilities::requires_auth::AuthenticationError> for AuthError {
    fn from(e: crate::utilities::requires_auth::AuthenticationError) -> Self {
        AuthError { message: e.to_string() }
    }
}
impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.message.as_str())
    }
}

#[derive(Union)]
pub enum PostMutationResult {
    PostType(PostType),
    DbError(DbErr),
    AuthError(AuthError),
}

#[derive(Default)]
pub struct PostMutation;

trait PostMutations {
    async fn add_post(&self, ctx: &Context<'_>, title: String, body: String) -> Result<PostMutationResult>;
}

impl RequiresAuth for PostMutation {}

#[Object]
impl PostMutations for PostMutation {
    async fn add_post(&self, ctx: &Context<'_>, title: String, body: String) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let post = posts::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(title),
            body: ActiveValue::set(body),
            user_id: ActiveValue::set(user.id),
            ..Default::default()
        };

        let res = match Posts::insert(post).exec(db).await {
            Ok(res) => res,
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbErr {
                    message: e.to_string(),
                }));
            }
        };
        
        let p = match Posts::find_by_id(res.last_insert_id).one(db).await {
            Ok(Some(p)) => p,
            Ok(None) => {
                return Ok(PostMutationResult::DbError(DbErr {
                    message: "Post not found".to_string(),
                }));
            }
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbErr {
                    message: e.to_string(),
                }));
            }
        };

        Ok(PostMutationResult::PostType(PostType {
            id: p.id,
            title: p.title,
            body: p.body,
            published_at: p.published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }))
    }
}
