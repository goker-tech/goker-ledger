use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::services::timeline::Timeline;
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct TimelineQuery {
    pub wallet: String,
    pub since: Option<i64>,
}

pub async fn get_timeline(
    State(state): State<AppState>,
    Query(query): Query<TimelineQuery>,
) -> AppResult<Json<Timeline>> {
    // Fetch fills and funding
    let fills = state
        .ingestion_service
        .fetch_all_fills(&query.wallet, query.since)
        .await?;

    let funding = state
        .ingestion_service
        .fetch_all_funding(&query.wallet, query.since)
        .await?;

    // Build timeline
    let timeline = state
        .timeline_service
        .build_timeline(&query.wallet, fills, funding)?;

    Ok(Json(timeline))
}
