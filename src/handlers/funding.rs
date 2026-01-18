use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::AppResult;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct FundingQuery {
    pub wallet: String,
    pub since: Option<i64>,
}

pub async fn get_funding(
    State(state): State<AppState>,
    Query(query): Query<FundingQuery>,
) -> AppResult<Json<Vec<Value>>> {
    let funding = state
        .ingestion_service
        .fetch_all_funding(&query.wallet, query.since)
        .await?;

    Ok(Json(funding))
}
