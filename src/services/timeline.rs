use bigdecimal::BigDecimal;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::str::FromStr;

use crate::error::AppResult;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "event_type", rename_all = "snake_case")]
pub enum TimelineEvent {
    Fill {
        timestamp: DateTime<Utc>,
        coin: String,
        side: String,
        size: BigDecimal,
        price: BigDecimal,
        fee: BigDecimal,
        realized_pnl: Option<BigDecimal>,
        tx_hash: Option<String>,
    },
    Funding {
        timestamp: DateTime<Utc>,
        coin: String,
        amount: BigDecimal,
        funding_rate: BigDecimal,
    },
    Liquidation {
        timestamp: DateTime<Utc>,
        coin: String,
        size: BigDecimal,
        price: BigDecimal,
        loss: BigDecimal,
    },
    Deposit {
        timestamp: DateTime<Utc>,
        amount: BigDecimal,
        token: String,
    },
    Withdrawal {
        timestamp: DateTime<Utc>,
        amount: BigDecimal,
        token: String,
    },
}

impl TimelineEvent {
    pub fn timestamp(&self) -> DateTime<Utc> {
        match self {
            TimelineEvent::Fill { timestamp, .. } => *timestamp,
            TimelineEvent::Funding { timestamp, .. } => *timestamp,
            TimelineEvent::Liquidation { timestamp, .. } => *timestamp,
            TimelineEvent::Deposit { timestamp, .. } => *timestamp,
            TimelineEvent::Withdrawal { timestamp, .. } => *timestamp,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Timeline {
    pub wallet: String,
    pub events: Vec<TimelineEvent>,
    pub from_timestamp: Option<DateTime<Utc>>,
    pub to_timestamp: Option<DateTime<Utc>>,
}

pub struct TimelineService;

impl TimelineService {
    pub fn new() -> Self {
        Self
    }

    /// Reconstructs a timeline from fills and funding payments
    pub fn build_timeline(
        &self,
        wallet: &str,
        fills: Vec<Value>,
        funding: Vec<Value>,
    ) -> AppResult<Timeline> {
        let mut events = Vec::new();

        // Process fills
        for fill in fills {
            if let Some(event) = self.parse_fill(&fill) {
                events.push(event);
            }
        }

        // Process funding payments
        for payment in funding {
            if let Some(event) = self.parse_funding(&payment) {
                events.push(event);
            }
        }

        // Sort by timestamp
        events.sort_by(|a, b| a.timestamp().cmp(&b.timestamp()));

        let from_timestamp = events.first().map(|e| e.timestamp());
        let to_timestamp = events.last().map(|e| e.timestamp());

        Ok(Timeline {
            wallet: wallet.to_string(),
            events,
            from_timestamp,
            to_timestamp,
        })
    }

    fn parse_fill(&self, fill: &Value) -> Option<TimelineEvent> {
        let timestamp = fill.get("time")
            .and_then(|t| t.as_i64())
            .map(|ts| DateTime::from_timestamp_millis(ts).unwrap_or_default())?;

        let coin = fill.get("coin").and_then(|c| c.as_str())?.to_string();
        let side = fill.get("side").and_then(|s| s.as_str())?.to_string();

        let size = fill.get("sz")
            .and_then(|s| s.as_str())
            .and_then(|s| BigDecimal::from_str(s).ok())?;

        let price = fill.get("px")
            .and_then(|p| p.as_str())
            .and_then(|p| BigDecimal::from_str(p).ok())?;

        let fee = fill.get("fee")
            .and_then(|f| f.as_str())
            .and_then(|f| BigDecimal::from_str(f).ok())
            .unwrap_or_default();

        let realized_pnl = fill.get("closedPnl")
            .and_then(|p| p.as_str())
            .and_then(|p| BigDecimal::from_str(p).ok());

        let tx_hash = fill.get("hash").and_then(|h| h.as_str()).map(String::from);

        Some(TimelineEvent::Fill {
            timestamp,
            coin,
            side,
            size,
            price,
            fee,
            realized_pnl,
            tx_hash,
        })
    }

    fn parse_funding(&self, payment: &Value) -> Option<TimelineEvent> {
        let timestamp = payment.get("time")
            .and_then(|t| t.as_i64())
            .map(|ts| DateTime::from_timestamp_millis(ts).unwrap_or_default())?;

        let coin = payment.get("coin").and_then(|c| c.as_str())?.to_string();

        let amount = payment.get("usdc")
            .and_then(|a| a.as_str())
            .and_then(|a| BigDecimal::from_str(a).ok())?;

        let funding_rate = payment.get("fundingRate")
            .and_then(|r| r.as_str())
            .and_then(|r| BigDecimal::from_str(r).ok())
            .unwrap_or_default();

        Some(TimelineEvent::Funding {
            timestamp,
            coin,
            amount,
            funding_rate,
        })
    }
}

impl Default for TimelineService {
    fn default() -> Self {
        Self::new()
    }
}
