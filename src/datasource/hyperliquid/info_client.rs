use async_trait::async_trait;
use reqwest::Client;
use serde_json::{json, Value};

use crate::datasource::DataSource;
use crate::error::{AppError, AppResult};

const MAX_ITEMS_PER_REQUEST: usize = 500;

#[derive(Clone)]
pub struct HyperliquidInfoClient {
    client: Client,
    base_url: String,
}

impl HyperliquidInfoClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: Client::new(),
            base_url: base_url.to_string(),
        }
    }

    async fn post(&self, payload: Value) -> AppResult<Value> {
        let response = self
            .client
            .post(&self.base_url)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_text = response.text().await.unwrap_or_default();
            return Err(AppError::ExternalApiError(format!(
                "Hyperliquid request failed: {}",
                error_text
            )));
        }

        let result: Value = response.json().await?;
        Ok(result)
    }

    /// Fetches all items with pagination handling (500 item limit)
    async fn fetch_paginated(
        &self,
        request_type: &str,
        wallet: &str,
        start_time: Option<i64>,
    ) -> AppResult<Vec<Value>> {
        let mut all_items = Vec::new();
        let mut current_start_time = start_time;

        loop {
            let mut payload = json!({
                "type": request_type,
                "user": wallet
            });

            if let Some(st) = current_start_time {
                payload["startTime"] = json!(st);
            }

            let response = self.post(payload).await?;

            let items = response
                .as_array()
                .cloned()
                .unwrap_or_default();

            let items_count = items.len();

            if items.is_empty() {
                break;
            }

            // Get the timestamp of the last item for pagination
            let last_timestamp = items
                .last()
                .and_then(|item| item.get("time"))
                .and_then(|t| t.as_i64());

            all_items.extend(items);

            // If we got fewer than 500 items, we've reached the end
            if items_count < MAX_ITEMS_PER_REQUEST {
                break;
            }

            // Update start time for next request
            if let Some(ts) = last_timestamp {
                current_start_time = Some(ts + 1);
            } else {
                break;
            }
        }

        Ok(all_items)
    }
}

#[async_trait]
impl DataSource for HyperliquidInfoClient {
    async fn get_fills(&self, wallet: &str, start_time: Option<i64>) -> AppResult<Vec<Value>> {
        self.fetch_paginated("userFills", wallet, start_time).await
    }

    async fn get_funding(&self, wallet: &str, start_time: Option<i64>) -> AppResult<Vec<Value>> {
        self.fetch_paginated("userFunding", wallet, start_time).await
    }

    async fn get_user_state(&self, wallet: &str) -> AppResult<Value> {
        let payload = json!({
            "type": "clearinghouseState",
            "user": wallet
        });
        self.post(payload).await
    }

    async fn get_all_mids(&self) -> AppResult<Value> {
        let payload = json!({
            "type": "allMids"
        });
        self.post(payload).await
    }
}
