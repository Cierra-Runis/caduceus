#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use actix_web::{App, HttpServer, web};
use server::{
    AppState,
    config::Config,
    database::Database,
    handler::ws::ProjectServer,
    repo::{project::MongoProjectRepo, team::MongoTeamRepo, user::MongoUserRepo},
    services::{project::ProjectService, team::TeamService, user::UserService},
    storage::{InMemoryObjectStore, MinioObjectStore, ObjectStore, ProjectStore},
};
use std::{env, io, sync::Arc};
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

    // Object storage for a project's blobs and Y.Doc snapshot, laid out under
    // `projects/{id}/...`. Falls back to an in-memory backend when unconfigured
    // so a checkout runs; production configures MinIO/S3 via `storage` in the
    // config. `ProjectStore` owns the key layout on top of the raw backend.
    let backend: Arc<dyn ObjectStore> = match &config.storage {
        Some(cfg) => Arc::new(
            MinioObjectStore::new(
                &cfg.endpoint,
                &cfg.region,
                &cfg.bucket,
                &cfg.access_key,
                &cfg.secret_key,
            )
            .expect("failed to init object storage"),
        ),
        None => Arc::new(InMemoryObjectStore::new()),
    };
    let store = ProjectStore::new(backend);

    // Create ProjectServer instance (actor-less implementation). It owns a repo
    // handle and the object store so collaboration rooms can persist the CRDT
    // Y.Doc (snapshot + projection) and text.
    let ws_config = config.ws.clone();
    let project_server =
        ProjectServer::new(project_repo.clone(), ws_config.clone(), store.clone());

    let jwt_secret = config.jwt_secret.clone();
    let address = config.address.clone();

    let factory = move || {
        let cors = config.cors();

        App::new()
            .wrap(cors)
            .app_data(data.clone())
            .app_data(web::Data::new(project_server.clone()))
            .app_data(web::Data::new(ws_config.clone()))
            .app_data(web::Data::new(store.clone()))
            // Server-side tinymist/LSP config. `None` only when this deployment
            // has no tinymist binaries configured (a bare checkout / CI) — not a
            // feature toggle: server-side compilation is the intended path,
            // replacing the client WASM compiler. The ws handshake reads it to
            // resolve a project's worker binary.
            .app_data(web::Data::new(config.lsp.clone()))
            .configure(|cfg| server::routes::configure(cfg, jwt_secret.clone()))
            .wrap(actix_web::middleware::Logger::default())
    };

    let mut server = HttpServer::new(factory);

    for addr in address {
        server = server.bind(addr)?;
    }

    server.run().await?;

    Ok(())
}
