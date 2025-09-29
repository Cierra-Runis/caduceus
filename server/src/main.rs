#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use actix::prelude::*;
use actix_web::{web, App, HttpServer};
use server::{
    config::Config,
    database::Database,
    handler::{self, ws::ProjectServer},
    middleware::jwt::JwtMiddleware,
    repo::{project::MongoProjectRepo, team::MongoTeamRepo, user::MongoUserRepo},
    services::{project::ProjectService, team::TeamService, user::UserService},
    AppState,
};
use std::{env, io};
use tracing_subscriber::fmt;

#[cfg_attr(coverage_nightly, coverage(off))]
#[actix_web::main]
async fn main() -> io::Result<()> {
    fmt::init();

    let env = env::var("APP_ENV").unwrap_or("dev".to_string());

    let config = Config::load(&format!("./config/{env}.yaml")).expect("Failed to load config");

    let database = Database::new(&config.mongo_uri, &config.db_name)
        .await
        .expect("Failed to connect to database");

    let user_repo = MongoUserRepo {
        collection: database.db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: database.db.collection("teams"),
    };
    let project_repo = MongoProjectRepo {
        collection: database.db.collection("projects"),
    };

    let data = web::Data::new(AppState {
        user_service: UserService {
            user_repo: user_repo.clone(),
            team_repo: team_repo.clone(),
            project_repo: project_repo.clone(),
            secret: config.jwt_secret.clone(),
        },
        team_service: TeamService {
            team_repo: team_repo.clone(),
            user_repo: user_repo.clone(),
            project_repo: project_repo.clone(),
        },
        project_service: ProjectService {
            project_repo: project_repo.clone(),
            user_repo: user_repo.clone(),
            team_repo: team_repo.clone(),
        },
    });

    // start ProjectServer actor and obtain its Addr to be stored in app data
    let project_server_addr = ProjectServer::new().start();
    let server_tx = handler::ws::ProjectServerHandle::new(project_server_addr.clone());

    let jwt_secret = config.jwt_secret.clone();
    let address = config.address.clone();

    let factory = move || {
        let cors = config.cors();

        App::new()
            .wrap(cors)
            .app_data(data.clone())
            .app_data(web::Data::new(server_tx.clone()))
            .route("/api/health", web::get().to(handler::health::health))
            .route("/api/register", web::post().to(handler::user::register))
            .route("/api/login", web::post().to(handler::user::login))
            .route("/api/logout", web::post().to(handler::user::logout))
            .service(
                web::scope("/api")
                    .wrap(JwtMiddleware::new(jwt_secret.clone()))
                    .route("/team", web::post().to(handler::team::create))
                    .route("/team/projects", web::get().to(handler::team::projects))
                    .route("/project", web::post().to(handler::project::create))
                    .route("/project/{id}", web::get().to(handler::project::find_by_id))
                    .service(
                        web::scope("/user")
                            .route("/me", web::get().to(handler::user::me))
                            .route("/teams", web::get().to(handler::user::teams))
                            .route("/projects", web::get().to(handler::user::projects)),
                    ),
            )
            .service(
                web::scope("/ws")
                    .wrap(JwtMiddleware::new(jwt_secret.clone()))
                    .route("/project/{id}", web::get().to(handler::ws::ws)),
            )
            .wrap(actix_web::middleware::Logger::default())
    };

    let mut server = HttpServer::new(factory);

    for addr in address {
        server = server.bind(addr)?;
    }

    server.run().await?;

    Ok(())
}
