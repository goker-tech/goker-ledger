pub mod hyperliquid;

use async_trait::async_trait;
use serde_json::Value;

use crate::error::AppResult;

/// Trait for data sources that provide trading history
#[async_trait]
pub trait DataSource: Send + Sync {
    /// Get user fills with pagination support
    async fn get_fills(&self, wallet: &str, start_time: Option<i64>) -> AppResult<Vec<Value>>;

    /// Get user funding payments with pagination support
    async fn get_funding(&self, wallet: &str, start_time: Option<i64>) -> AppResult<Vec<Value>>;

    /// Get user's current state (positions, balances)
    async fn get_user_state(&self, wallet: &str) -> AppResult<Value>;

    /// Get all available mid prices
    async fn get_all_mids(&self) -> AppResult<Value>;
}
