mod config;
mod db;
mod fetcher;
mod index;
mod models;
mod routes;

use axum::Router;
use routes::AppState;
use std::sync::Arc;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| "info".into()),
        )
        .init();

    let cfg = config::StocksConfig::load().expect("Failed to load stocks.toml");
    tracing::info!(
        "Loaded {} index symbols, {} total",
        cfg.index_symbols().len(),
        cfg.all_symbols().len()
    );

    let pool = db::init_pool()
        .await
        .expect("Failed to initialize database");
    let config = Arc::new(cfg);

    // Spawn background fetcher.
    fetcher::spawn(pool.clone(), config.clone());

    let state = AppState { pool, config };

    let api = Router::new()
        .route("/api/health", axum::routing::get(routes::health))
        .route("/api/index", axum::routing::get(routes::get_index))
        .route(
            "/api/index/history",
            axum::routing::get(routes::get_index_history),
        )
        .route("/api/stocks", axum::routing::get(routes::get_stocks))
        .route(
            "/api/stocks/{symbol}",
            axum::routing::get(routes::get_stock),
        )
        .route("/api/sectors", axum::routing::get(routes::get_sectors))
        .route("/api/config", axum::routing::get(routes::get_config))
        .with_state(state);

    // In production, serve static files from /app/dist; in dev, Vite proxies.
    let app = api
        .fallback_service(ServeDir::new("dist"))
        .layer(CorsLayer::permissive());

    let addr = std::env::var("BIND_ADDR").unwrap_or_else(|_| "0.0.0.0:8080".to_string());
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
