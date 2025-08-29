use anyhow::Result;
use axum::serve;
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
    tracing_subscriber::fmt::init();

    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    let config = Config::load(env, "config".to_string())?;
    info!("Configuration loaded successfully");

    let database = Database::new(&config.mongo_uri, &config.db_name).await?;
    info!("Connected to MongoDB successfully");

    let app_state = routes::AppState {
        database,
        config: config.clone(),
    };

    let app = create_routes(app_state).layer(CorsLayer::permissive());
    // .fallback(|| async { (StatusCode::NOT_FOUND, Json("Not Found")) });

    let listener = TcpListener::bind(&config.address)
        .await
        .map_err(|e| anyhow::anyhow!("Failed to bind to address {}: {}", config.address, e))?;

    let addr = listener.local_addr()?;
    info!("Starting server on {}", addr);

    serve(listener, app).await?;

    Ok(())
}
