use serde_json::Value;
use std::sync::Arc;

use crate::datasource::DataSource;
use crate::error::AppResult;

pub struct IngestionService {
    datasource: Arc<dyn DataSource>,
}

impl IngestionService {
    pub fn new(datasource: Arc<dyn DataSource>) -> Self {
        Self { datasource }
    }

    /// Fetches all fills for a wallet, handling the 500 item pagination limit
    pub async fn fetch_all_fills(&self, wallet: &str, since: Option<i64>) -> AppResult<Vec<Value>> {
        tracing::info!("Fetching fills for wallet: {}", wallet);
        let fills = self.datasource.get_fills(wallet, since).await?;
        tracing::info!("Fetched {} fills", fills.len());
        Ok(fills)
    }

    /// Fetches all funding payments for a wallet
    pub async fn fetch_all_funding(&self, wallet: &str, since: Option<i64>) -> AppResult<Vec<Value>> {
        tracing::info!("Fetching funding for wallet: {}", wallet);
        let funding = self.datasource.get_funding(wallet, since).await?;
        tracing::info!("Fetched {} funding payments", funding.len());
        Ok(funding)
    }

    /// Fetches current user state (positions, balances)
    pub async fn fetch_user_state(&self, wallet: &str) -> AppResult<Value> {
        self.datasource.get_user_state(wallet).await
    }

    /// Fetches current mid prices for all assets
    pub async fn fetch_all_mids(&self) -> AppResult<Value> {
        self.datasource.get_all_mids().await
    }
}
