use anyhow::Result;
use axum::{http::StatusCode, response::Json, serve};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tower_http::cors::CorsLayer;
use tracing::info;

mod config;
mod database;
mod error;
mod handlers;
mod middleware;
mod models;
mod routes;
mod services;

use config::Config;
use database::Database;
use routes::create_routes;

#[tokio::main]
async fn main() -> Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    // Load configuration
    let config = Config::load(env)?;
    info!("Configuration loaded successfully");

    // Connect to database
    let database = Database::new(&config.mongo_uri, &config.db_name).await?;
    info!("Connected to MongoDB successfully");

    // Create application state
    let app_state = routes::AppState {
        database,
        config: config.clone(),
    };

    // Build our application with routes
    let app = create_routes(app_state)
        .layer(CorsLayer::permissive())
        .fallback(|| async { (StatusCode::NOT_FOUND, Json("Not Found")) });

    // Parse address
    let addr: SocketAddr = config
        .address
        .parse()
        .map_err(|e| anyhow::anyhow!("Invalid address format: {}", e))?;

    info!("Starting server on {}", addr);

    // Create TCP listener
    let listener = TcpListener::bind(&addr).await?;

    // Start server
    serve(listener, app).await?;

    Ok(())
}
