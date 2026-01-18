use axum::{
    http::{header, Method},
    routing::get,
    Router,
};
use std::env;
use std::sync::Arc;
use tower_http::cors::{Any, CorsLayer};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod datasource;
mod error;
mod handlers;
mod services;

use datasource::hyperliquid::HyperliquidInfoClient;
use datasource::DataSource;
use services::ingestion::IngestionService;
use services::pnl_calculator::PnlCalculator;
use services::timeline::TimelineService;

#[derive(Clone)]
pub struct AppState {
    pub ingestion_service: Arc<IngestionService>,
    pub timeline_service: Arc<TimelineService>,
    pub pnl_calculator: Arc<PnlCalculator>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize tracing
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "goker_ledger=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    // Load configuration from environment
    dotenvy::dotenv().ok();

    let hyperliquid_info_url = env::var("HYPERLIQUID_INFO_URL")
        .unwrap_or_else(|_| "https://api.hyperliquid.xyz/info".to_string());

    let server_host = env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let server_port = env::var("SERVER_PORT").unwrap_or_else(|_| "8081".to_string());

    // Initialize data source
    let datasource: Arc<dyn DataSource> =
        Arc::new(HyperliquidInfoClient::new(&hyperliquid_info_url));

    // Initialize services
    let ingestion_service = Arc::new(IngestionService::new(datasource));
    let timeline_service = Arc::new(TimelineService::new());
    let pnl_calculator = Arc::new(PnlCalculator::new());

    // Create app state
    let state = AppState {
        ingestion_service,
        timeline_service,
        pnl_calculator,
    };

    // Build CORS layer
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET])
        .allow_headers([header::CONTENT_TYPE]);

    // Build router
    let app = Router::new()
        .route("/health", get(|| async { "OK" }))
        .route("/timeline", get(handlers::timeline::get_timeline))
        .route("/pnl", get(handlers::pnl::get_pnl_summary))
        .route("/pnl/daily", get(handlers::pnl::get_daily_pnl))
        .route("/fills", get(handlers::fills::get_fills))
        .route("/funding", get(handlers::funding::get_funding))
        .layer(cors)
        .with_state(state);

    // Start server
    let addr = format!("{}:{}", server_host, server_port);
    tracing::info!("Starting Ledger API server on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
