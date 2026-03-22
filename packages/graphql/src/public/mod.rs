mod queries;
mod rate_limiter;
mod types;

pub use queries::{PublicApiKey, PublicQueryRoot};

use async_graphql::{EmptyMutation, EmptySubscription, Schema};
use rate_limiter::{BudgetLimiterFactory, SlidingBudget};
use sea_orm::DatabaseConnection;
use std::sync::Arc;

use crate::utilities::MarkdownCache;

pub type PublicSchema = Schema<PublicQueryRoot, EmptyMutation, EmptySubscription>;

pub fn build_public_schema(db: DatabaseConnection, markdown_cache: MarkdownCache) -> PublicSchema {
    let max_complexity = std::env::var("PUBLIC_MAX_COMPLEXITY")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(200_usize);
    let max_depth = std::env::var("PUBLIC_MAX_DEPTH")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(5_usize);
    let budget = std::env::var("PUBLIC_COMPLEXITY_BUDGET")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(2000_usize);
    let window_secs = std::env::var("PUBLIC_COMPLEXITY_WINDOW_SECS")
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(60_u64);

    let limiter = Arc::new(SlidingBudget::new(window_secs, budget));

    Schema::build(PublicQueryRoot::default(), EmptyMutation, EmptySubscription)
        .data(db)
        .data(markdown_cache)
        .limit_complexity(max_complexity)
        .limit_depth(max_depth)
        .extension(BudgetLimiterFactory(limiter))
        .finish()
}
