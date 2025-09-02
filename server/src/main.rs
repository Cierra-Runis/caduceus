#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use std::env;
use tracing_subscriber::fmt;

mod config;
mod database;
mod handler;
mod models;
mod repo;
mod services;

use crate::{repo::user::MongoUserRepo, services::user::UserService};
use config::Config;
use database::Database;

pub struct AppState {
    pub database: Database,
    pub config: Config,
    pub user_service: UserService<MongoUserRepo>,
}

#[cfg_attr(coverage_nightly, coverage(off))]
#[tokio::main]
async fn main() -> Result<()> {
    fmt::init();

    let env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    let config = Config::load(env, "config".to_string())?;

    let database = Database::new(&config.mongo_uri, &config.db_name).await?;

    let user = UserService {
        repo: MongoUserRepo {
            collection: database.db.collection("users"),
        },
        secret: config.jwt_secret.clone(),
    };

    let data = web::Data::new(AppState {
        database,
        config,
        user_service,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .wrap(actix_web::middleware::Logger::default())
            .service(
                web::scope("/api")
                    .route("/health", web::get().to(handler::health::health))
                    .route("/user", web::post().to(handler::user::register))
                    .route("/user", web::get().to(handler::user::login)),
            )
    })
    .bind(("127.0.0.1", 8080))?
    .bind(("[::1]", 8080))?
    .run()
    .await?;

    Ok(())
}
