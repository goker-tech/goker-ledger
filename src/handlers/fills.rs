use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;
use serde_json::Value;

use crate::error::AppResult;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct FillsQuery {
    pub wallet: String,
    pub since: Option<i64>,
}

pub async fn get_fills(
    State(state): State<AppState>,
    Query(query): Query<FillsQuery>,
) -> AppResult<Json<Vec<Value>>> {
    let fills = state
        .ingestion_service
        .fetch_all_fills(&query.wallet, query.since)
        .await?;

    Ok(Json(fills))
}
