#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

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
use tokio::spawn;
use tokio::try_join;
use tracing_subscriber::fmt;

#[cfg_attr(coverage_nightly, coverage(off))]
#[tokio::main]
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

    let (project_server, server_tx) = ProjectServer::new();
    let project_server = spawn(project_server.run());

    let http_server = HttpServer::new(move || {
        App::new()
            .app_data(data.clone())
            .app_data(web::Data::new(server_tx.clone()))
            .route("/api/health", web::get().to(handler::health::health))
            .route("/api/register", web::post().to(handler::user::register))
            .route("/api/login", web::post().to(handler::user::login))
            .route("/api/logout", web::post().to(handler::user::logout))
            .service(
                web::scope("/api")
                    .wrap(JwtMiddleware::new(config.jwt_secret.clone()))
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
            .route("ws", web::get().to(handler::ws::ws))
            .wrap(actix_web::middleware::Logger::default())
    })
    .bind(("127.0.0.1", 8080))?
    .bind(("[::1]", 8080))?
    .run();

    try_join!(http_server, async move { project_server.await.unwrap() })?;

    Ok(())
}
