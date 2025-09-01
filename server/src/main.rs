#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use actix_web::{web, App, HttpServer};
use anyhow::Result;

mod config;
mod database;
mod handler;
mod models;
mod repo;
mod services;

use config::Config;
use database::Database;

use crate::{repo::user::MongoUserRepo, services::user::UserService};

pub struct AppState {
    pub database: Database,
    pub config: Config,
    pub user: UserService<MongoUserRepo>,
}

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt::init();

    let env = std::env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    let config = Config::load(env, "config".to_string())?;

    let database = Database::new(&config.mongo_uri, &config.db_name).await?;

    let user = UserService {
        repo: MongoUserRepo {
            collection: database.db.collection("users"),
        },
    };

    let data = web::Data::new(AppState {
        database,
        config,
        user,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .service(web::scope("/api").route("/user", web::post().to(handler::user::create_user)))
    })
    .bind(("127.0.0.1", 8080))?
    .run()
    .await?;

    Ok(())
}
