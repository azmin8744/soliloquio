mod queries;
mod types;

pub use queries::{PublicApiKey, PublicQueryRoot};

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use sea_orm::DatabaseConnection;

use crate::utilities::MarkdownCache;

pub type PublicSchema = Schema<PublicQueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_public_schema(db: DatabaseConnection, markdown_cache: MarkdownCache) -> PublicSchema {
    Schema::build(PublicQueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(db)
        .data(markdown_cache)
        .finish()
}
