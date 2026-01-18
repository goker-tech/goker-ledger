use axum::{
    extract::{Query, State},
    Json,
};
use serde::Deserialize;

use crate::error::AppResult;
use crate::services::pnl_calculator::{DailyPnl, PnlSummary};
use crate::AppState;

#[derive(Debug, Deserialize)]
pub struct PnlQuery {
    pub wallet: String,
    pub since: Option<i64>,
}

pub async fn get_pnl_summary(
    State(state): State<AppState>,
    Query(query): Query<PnlQuery>,
) -> AppResult<Json<PnlSummary>> {
    // Fetch data
    let fills = state
        .ingestion_service
        .fetch_all_fills(&query.wallet, query.since)
        .await?;

    let funding = state
        .ingestion_service
        .fetch_all_funding(&query.wallet, query.since)
        .await?;

    let user_state = state
        .ingestion_service
        .fetch_user_state(&query.wallet)
        .await?;

    // Build timeline
    let timeline = state
        .timeline_service
        .build_timeline(&query.wallet, fills, funding)?;

    // Calculate unrealized PnL
    let unrealized_pnl = state.pnl_calculator.calculate_unrealized_from_state(&user_state);

    // Calculate PnL summary
    let summary = state
        .pnl_calculator
        .calculate_summary(&query.wallet, &timeline, unrealized_pnl);

    Ok(Json(summary))
}

pub async fn get_daily_pnl(
    State(state): State<AppState>,
    Query(query): Query<PnlQuery>,
) -> AppResult<Json<Vec<DailyPnl>>> {
    // Fetch data
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

    // Calculate daily PnL
    let daily = state.pnl_calculator.calculate_daily(&timeline);

    Ok(Json(daily))
}
