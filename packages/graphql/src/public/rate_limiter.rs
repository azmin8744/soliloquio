use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};

use async_graphql::extensions::{Extension, ExtensionContext, ExtensionFactory, NextValidation};
use async_graphql::{ServerError, ValidationResult};
use dashmap::DashMap;

use crate::public::PublicApiKey;

pub struct SlidingBudget {
    windows: DashMap<String, VecDeque<(Instant, usize)>>,
    window: Duration,
    max_budget: usize,
}

impl SlidingBudget {
    pub fn new(window_secs: u64, max_budget: usize) -> Self {
        Self { windows: DashMap::new(), window: Duration::from_secs(window_secs), max_budget }
    }

    /// Returns false if budget exceeded (does NOT deduct in that case)
    pub fn check_and_deduct(&self, key: &str, cost: usize) -> bool {
        let now = Instant::now();
        let cutoff = now - self.window;
        let mut entry = self.windows.entry(key.to_string()).or_default();
        entry.retain(|(t, _)| *t > cutoff);
        let used: usize = entry.iter().map(|(_, c)| c).sum();
        if used + cost <= self.max_budget {
            entry.push_back((now, cost));
            true
        } else {
            false
        }
    }
}

pub struct BudgetLimiterFactory(pub Arc<SlidingBudget>);

impl ExtensionFactory for BudgetLimiterFactory {
    fn create(&self) -> Arc<dyn Extension> {
        Arc::new(BudgetLimiterExt(self.0.clone()))
    }
}

struct BudgetLimiterExt(Arc<SlidingBudget>);

#[async_graphql::async_trait::async_trait]
impl Extension for BudgetLimiterExt {
    async fn validation(
        &self,
        ctx: &ExtensionContext<'_>,
        next: NextValidation<'_>,
    ) -> Result<ValidationResult, Vec<ServerError>> {
        let result = next.run(ctx).await?;
        if let Ok(key) = ctx.data::<PublicApiKey>() {
            if !self.0.check_and_deduct(&key.0, result.complexity) {
                return Err(vec![ServerError::new(
                    "Rate limit exceeded: complexity budget exhausted",
                    None,
                )]);
            }
        }
        Ok(result)
    }
}
