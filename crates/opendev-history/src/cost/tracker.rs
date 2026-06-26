use std::sync::Mutex;

use chrono::Utc;

use super::types::{CostCall, ModelPricing, TokenUsage};

fn calculate_cost(usage: &TokenUsage, pricing: &ModelPricing) -> f64 {
    let input_cost = usage.prompt_tokens as f64 * pricing.input_per_million / 1_000_000.0;
    let output_cost = usage.completion_tokens as f64 * pricing.output_per_million / 1_000_000.0;
    let cache_read_cost = usage.cache_read_tokens as f64
        * pricing.cache_read_per_million.unwrap_or(pricing.input_per_million * 0.1)
        / 1_000_000.0;
    let cache_write_cost = usage.cache_write_tokens as f64
        * pricing.cache_write_per_million.unwrap_or(pricing.input_per_million * 1.25)
        / 1_000_000.0;
    input_cost + output_cost + cache_read_cost + cache_write_cost
}

#[derive(Default)]
struct CostState {
    total_rmb: f64,
    daily_rmb: f64,
    monthly_rmb: f64,
    records: Vec<CostCall>,
    pool: Option<sqlx::SqlitePool>,
}

impl std::fmt::Debug for CostTracker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CostTracker").finish_non_exhaustive()
    }
}

pub struct CostTracker {
    state: Mutex<CostState>,
}

impl Default for CostTracker {
    fn default() -> Self {
        Self { state: Mutex::new(CostState::default()) }
    }
}

impl CostTracker {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_pool(self, pool: sqlx::SqlitePool) -> Self {
        if let Ok(mut state) = self.state.lock() {
            state.pool = Some(pool);
        }
        self
    }

    pub fn record(
        &self,
        provider: &str,
        model: &str,
        usage: &TokenUsage,
        pricing: &ModelPricing,
    ) -> f64 {
        let cost = calculate_cost(usage, pricing);
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.total_rmb += cost;
        state.daily_rmb += cost;
        state.monthly_rmb += cost;
        state.records.push(CostCall {
            provider: provider.into(),
            model: model.into(),
            prompt_tokens: usage.prompt_tokens,
            completion_tokens: usage.completion_tokens,
            cache_read_tokens: usage.cache_read_tokens,
            cache_write_tokens: usage.cache_write_tokens,
            cost_rmb: cost,
        });

        if let Some(pool) = &state.pool {
            let pool = pool.clone();
            let model = model.to_string();
            let provider = provider.to_string();
            let session_id = "".to_string();
            let now = Utc::now();
            let tokens_p = usage.prompt_tokens as i64;
            let tokens_c = usage.completion_tokens as i64;
            let tokens_cr = usage.cache_read_tokens as i64;
            let tokens_cw = usage.cache_write_tokens as i64;
            let tokens_t = usage.thinking_tokens.map(|t| t as i64);
            let ts = now.to_rfc3339();

            // Try to persist within a current runtime, or silently skip if none
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                handle.spawn(async move {
                    let sql = "INSERT INTO cost_records \
                        (session_id, model, provider, prompt_tokens, completion_tokens, \
                         cache_read_tokens, cache_write_tokens, thinking_tokens, cost_rmb, created_at) \
                        VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)";
                    if let Err(e) = sqlx::query(sql)
                        .bind(&session_id)
                        .bind(&model)
                        .bind(&provider)
                        .bind(tokens_p)
                        .bind(tokens_c)
                        .bind(tokens_cr)
                        .bind(tokens_cw)
                        .bind(tokens_t)
                        .bind(cost)
                        .bind(&ts)
                        .execute(&pool)
                        .await
                    {
                        tracing::warn!("failed to persist cost record: {e}");
                    }
                });
            }
        }

        cost
    }

    pub fn total_rmb(&self) -> f64 {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).total_rmb
    }

    pub fn daily_rmb(&self) -> f64 {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).daily_rmb
    }

    pub fn monthly_rmb(&self) -> f64 {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).monthly_rmb
    }

    pub fn reset_periods(&self) {
        let mut state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.daily_rmb = 0.0;
        state.monthly_rmb = 0.0;
    }

    pub fn records(&self) -> Vec<CostCall> {
        self.state.lock().unwrap_or_else(|e| e.into_inner()).records.clone()
    }

    pub fn recent_records(&self, n: usize) -> Vec<CostCall> {
        let state = self.state.lock().unwrap_or_else(|e| e.into_inner());
        state.records.iter().rev().take(n).cloned().collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cost_calculation() {
        let tracker = CostTracker::new();
        let pricing = ModelPricing {
            input_per_million: 3.0,
            output_per_million: 15.0,
            cache_read_per_million: Some(0.3),
            cache_write_per_million: None,
        };
        let usage = TokenUsage {
            prompt_tokens: 1_000_000,
            completion_tokens: 500_000,
            cache_read_tokens: 100_000,
            cache_write_tokens: 0,
            thinking_tokens: None,
        };
        let cost = tracker.record("anthropic", "claude", &usage, &pricing);
        assert!((cost - (3.0 + 7.5 + 0.03)).abs() < 0.001);
        assert!((tracker.total_rmb() - cost).abs() < 0.001);
    }
}
