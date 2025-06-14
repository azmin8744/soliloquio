use async_graphql::{Context, InputObject, Object, Result, SimpleObject, Union};
use sea_orm::*;
use std::fmt;
use models::{prelude::*, *};
use sea_orm::entity::prelude::Uuid;
use crate::types::post::{Post as PostType, DeletedPost};
use crate::utilities::requires_auth::RequiresAuth;

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
    ChangedPost(PostType),
    DeletedPost(DeletedPost),
    DbError(DbErr),
    AuthError(AuthError),
}

#[derive(InputObject)]
struct AddPostInput {
    title: String,
    body: String,
    is_published: Option<bool>,
}

#[derive(InputObject)]
struct UpdatePostInput {
    id: Uuid,
    title: String,
    body: String,
    is_published: Option<bool>,
}

#[derive(InputObject)]
struct DeletePostInput {
    id: Uuid,
}

#[derive(Default)]
pub struct PostMutation;

trait PostMutations {
    async fn add_post(&self, ctx: &Context<'_>, new_post: AddPostInput) -> Result<PostMutationResult>;
    async fn update_post(&self, ctx: &Context<'_>, post: UpdatePostInput) -> Result<PostMutationResult>;
    async fn delete_post(&self, ctx: &Context<'_>, post: DeletePostInput) -> Result<PostMutationResult>;
}

impl RequiresAuth for PostMutation {}

#[Object]
impl PostMutations for PostMutation {
    async fn add_post(&self, ctx: &Context<'_>, new_post: AddPostInput) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let is_published = new_post.is_published.unwrap_or(false);
        let first_published_at = if is_published { 
            Some(chrono::Utc::now().naive_utc())
        } else {
            None
        };
        
        let post = posts::ActiveModel {
            id: ActiveValue::set(Uuid::new_v4()),
            title: ActiveValue::set(new_post.title),
            body: ActiveValue::set(new_post.body),
            user_id: ActiveValue::set(user.id),
            is_published: ActiveValue::set(is_published),
            first_published_at: ActiveValue::set(first_published_at),
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

        Ok(PostMutationResult::ChangedPost(PostType {
            id: p.id,
            title: p.title,
            body: p.body,
            is_published: p.is_published,
            first_published_at: p.first_published_at,
            created_at: p.created_at,
            updated_at: p.updated_at,
        }))
    }

    async fn update_post(&self, ctx: &Context<'_>, post: UpdatePostInput) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let _user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let mut post_to_update = match Posts::find_by_id(post.id).one(db).await {
            Ok(Some(p)) => p.into_active_model(),
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

        post_to_update.title = ActiveValue::set(post.title);
        post_to_update.body = ActiveValue::set(post.body);
        post_to_update.updated_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
        
        // Handle publication status change if provided
        if let Some(is_published) = post.is_published {
            post_to_update.is_published = ActiveValue::set(is_published);
            
            // Set first_published_at if being published for the first time
            if is_published {
                let post_model = Posts::find_by_id(post.id).one(db).await.unwrap().unwrap();
                if post_model.first_published_at.is_none() {
                    post_to_update.first_published_at = ActiveValue::set(Some(chrono::Utc::now().naive_utc()));
                }
            }
        }

        let _res = match Posts::update(post_to_update).exec(db).await {
            Ok(p) => {
                return Ok(PostMutationResult::ChangedPost(PostType {
                    id: p.id,
                    title: p.title,
                    body: p.body,
                    is_published: p.is_published,
                    first_published_at: p.first_published_at,
                    created_at: p.created_at,
                    updated_at: p.updated_at,
                }))
            },
            Err(e) => {
                return Ok(PostMutationResult::DbError(DbErr {
                    message: e.to_string(),
                }));
            }
        };
    }

    async fn delete_post(&self, ctx: &Context<'_>, post: DeletePostInput) -> Result<PostMutationResult> {
        let db = ctx.data::<DatabaseConnection>().unwrap();

        let _user = match self.require_authenticate_as_user(ctx).await {
            Ok(user) => user,
            Err(e) => {
                return Ok(PostMutationResult::AuthError(AuthError {
                    message: e.to_string(),
                }));
            }
        };

        let post_to_delete = match Posts::find_by_id(post.id).one(db).await {
            Ok(Some(p)) => p.into_active_model(),
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

        match Posts::delete(post_to_delete.clone()).exec(db).await {
            Ok(_) => Ok(PostMutationResult::DeletedPost(DeletedPost { id: post_to_delete.id.unwrap() })),
            Err(e) => Ok(PostMutationResult::DbError(DbErr {
                message: e.to_string(),
            })),
        }
    }
}
