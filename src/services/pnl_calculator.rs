use bigdecimal::BigDecimal;
use chrono::{DateTime, Duration, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::str::FromStr;

use crate::services::timeline::{Timeline, TimelineEvent};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PnlSummary {
    pub wallet: String,
    pub period_start: DateTime<Utc>,
    pub period_end: DateTime<Utc>,
    pub realized_pnl: BigDecimal,
    pub unrealized_pnl: BigDecimal,
    pub total_pnl: BigDecimal,
    pub funding_pnl: BigDecimal,
    pub trading_fees: BigDecimal,
    pub net_pnl: BigDecimal,
    pub by_asset: HashMap<String, AssetPnl>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetPnl {
    pub coin: String,
    pub realized_pnl: BigDecimal,
    pub funding_pnl: BigDecimal,
    pub fees: BigDecimal,
    pub net_pnl: BigDecimal,
    pub trade_count: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyPnl {
    pub date: String,
    pub pnl: BigDecimal,
    pub cumulative_pnl: BigDecimal,
}

pub struct PnlCalculator;

impl PnlCalculator {
    pub fn new() -> Self {
        Self
    }

    /// Calculates PnL summary from timeline events
    pub fn calculate_summary(
        &self,
        wallet: &str,
        timeline: &Timeline,
        unrealized_pnl: BigDecimal,
    ) -> PnlSummary {
        let mut realized_pnl = BigDecimal::from(0);
        let mut funding_pnl = BigDecimal::from(0);
        let mut trading_fees = BigDecimal::from(0);
        let mut by_asset: HashMap<String, AssetPnl> = HashMap::new();

        for event in &timeline.events {
            match event {
                TimelineEvent::Fill {
                    coin,
                    fee,
                    realized_pnl: rpnl,
                    ..
                } => {
                    trading_fees = &trading_fees + fee;

                    let asset_pnl = by_asset.entry(coin.clone()).or_insert_with(|| AssetPnl {
                        coin: coin.clone(),
                        realized_pnl: BigDecimal::from(0),
                        funding_pnl: BigDecimal::from(0),
                        fees: BigDecimal::from(0),
                        net_pnl: BigDecimal::from(0),
                        trade_count: 0,
                    });

                    asset_pnl.fees = &asset_pnl.fees + fee;
                    asset_pnl.trade_count += 1;

                    if let Some(pnl) = rpnl {
                        realized_pnl = &realized_pnl + pnl;
                        asset_pnl.realized_pnl = &asset_pnl.realized_pnl + pnl;
                    }
                }
                TimelineEvent::Funding { coin, amount, .. } => {
                    funding_pnl = &funding_pnl + amount;

                    let asset_pnl = by_asset.entry(coin.clone()).or_insert_with(|| AssetPnl {
                        coin: coin.clone(),
                        realized_pnl: BigDecimal::from(0),
                        funding_pnl: BigDecimal::from(0),
                        fees: BigDecimal::from(0),
                        net_pnl: BigDecimal::from(0),
                        trade_count: 0,
                    });

                    asset_pnl.funding_pnl = &asset_pnl.funding_pnl + amount;
                }
                _ => {}
            }
        }

        // Calculate net PnL for each asset
        for asset_pnl in by_asset.values_mut() {
            asset_pnl.net_pnl =
                &asset_pnl.realized_pnl + &asset_pnl.funding_pnl - &asset_pnl.fees;
        }

        let total_pnl = &realized_pnl + &unrealized_pnl;
        let net_pnl = &total_pnl + &funding_pnl - &trading_fees;

        let period_start = timeline.from_timestamp.unwrap_or_else(Utc::now);
        let period_end = timeline.to_timestamp.unwrap_or_else(Utc::now);

        PnlSummary {
            wallet: wallet.to_string(),
            period_start,
            period_end,
            realized_pnl,
            unrealized_pnl,
            total_pnl,
            funding_pnl,
            trading_fees,
            net_pnl,
            by_asset,
        }
    }

    /// Calculates daily PnL breakdown
    pub fn calculate_daily(&self, timeline: &Timeline) -> Vec<DailyPnl> {
        let mut daily_map: HashMap<String, BigDecimal> = HashMap::new();

        for event in &timeline.events {
            let date = event.timestamp().format("%Y-%m-%d").to_string();

            let pnl = match event {
                TimelineEvent::Fill {
                    realized_pnl,
                    fee,
                    ..
                } => {
                    let rpnl = realized_pnl.clone().unwrap_or_default();
                    &rpnl - fee
                }
                TimelineEvent::Funding { amount, .. } => amount.clone(),
                TimelineEvent::Liquidation { loss, .. } => -loss.clone(),
                _ => BigDecimal::from(0),
            };

            let entry = daily_map.entry(date).or_insert_with(|| BigDecimal::from(0));
            *entry = &*entry + &pnl;
        }

        let mut daily_pnl: Vec<DailyPnl> = daily_map
            .into_iter()
            .map(|(date, pnl)| DailyPnl {
                date,
                pnl,
                cumulative_pnl: BigDecimal::from(0),
            })
            .collect();

        // Sort by date
        daily_pnl.sort_by(|a, b| a.date.cmp(&b.date));

        // Calculate cumulative PnL
        let mut cumulative = BigDecimal::from(0);
        for day in &mut daily_pnl {
            cumulative = &cumulative + &day.pnl;
            day.cumulative_pnl = cumulative.clone();
        }

        daily_pnl
    }

    /// Calculates unrealized PnL from current positions
    pub fn calculate_unrealized_from_state(&self, user_state: &serde_json::Value) -> BigDecimal {
        user_state
            .get("assetPositions")
            .and_then(|positions| positions.as_array())
            .map(|positions| {
                positions
                    .iter()
                    .filter_map(|p| {
                        p.get("position")
                            .and_then(|pos| pos.get("unrealizedPnl"))
                            .and_then(|pnl| pnl.as_str())
                            .and_then(|s| BigDecimal::from_str(s).ok())
                    })
                    .fold(BigDecimal::from(0), |acc, pnl| &acc + &pnl)
            })
            .unwrap_or_default()
    }
}

impl Default for PnlCalculator {
    fn default() -> Self {
        Self::new()
    }
}
