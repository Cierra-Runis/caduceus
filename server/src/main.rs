#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use actix_web::{web, App, HttpServer};
use anyhow::Result;
use std::env;
use tracing_subscriber::fmt;

use caduceus_server::{
    config::Config, database::Database, handler, middleware::jwt::JwtMiddleware,
    repo::{team::MongoTeamRepo, user::MongoUserRepo},
    services::{team::TeamService, user::UserService},
    AppState,
};

#[cfg_attr(coverage_nightly, coverage(off))]
#[tokio::main]
async fn main() -> Result<()> {
    fmt::init();

    let env = env::var("APP_ENV").unwrap_or_else(|_| "dev".to_string());

    let config = Config::load(&format!("./config/{env}.yaml"))?;

    let database = Database::new(&config.mongo_uri, &config.db_name).await?;

    let user_repo = MongoUserRepo {
        collection: database.db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: database.db.collection("teams"),
    };

    let user_service = UserService {
        user_repo: user_repo.clone(),
        secret: config.jwt_secret.clone(),
    };
    let team_service = TeamService {
        team_repo: team_repo.clone(),
        user_repo: user_repo.clone(),
    };

    let data = web::Data::new(AppState {
        database,
        config,
        user_service,
        team_service,
    });

    HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .route("/api/health", web::get().to(handler::health::health))
            .route("/api/register", web::post().to(handler::user::register))
            .route("/api/login", web::post().to(handler::user::login))
            .route("/api/logout", web::post().to(handler::user::logout))
            .service(
                web::scope("/api")
                    .wrap(JwtMiddleware)
                    .route("/team", web::post().to(handler::team::create)),
            )
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .bind(("[::1]", 8080))?
    .run()
    .await?;

    Ok(())
}
