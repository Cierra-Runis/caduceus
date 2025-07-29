mod controller;
mod model;
mod router;

use dotenv::dotenv;
use mongodb::Client;
use std::env;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[tokio::main]
async fn main() {
    dotenv().ok();
    let mongo_uri = env::var("MONGO_URI").expect("MONGO_URI must be set in .env file");
    let client = Client::with_uri_str(mongo_uri)
        .await
        .expect("Failed to initialize MongoDB client");
    let db_name = env::var("DB_NAME").expect("DB_NAME must be set in .env file");
    let database = client.database(&db_name);

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    tracing::debug!("Listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, router::app(database)).await.unwrap();
}
