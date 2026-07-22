#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

use actix_web::{App, HttpServer, web};
use server::{
    AppState,
    config::{Config, StorageConfig},
    database::Database,
    handler::ws::ProjectServer,
    repo::{
        asset::{AssetStoreKind, GridFsAssetStore, S3AssetStore},
        project::MongoProjectRepo,
        team::MongoTeamRepo,
        user::MongoUserRepo,
    },
    services::{
        asset::AssetService, project::ProjectService, team::TeamService, user::UserService,
    },
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

    // Resolve the binary-asset backend from config: GridFS by default (no extra
    // infrastructure), or an S3-compatible object store when configured.
    let asset_store = match &config.storage {
        StorageConfig::GridFs => AssetStoreKind::GridFs(GridFsAssetStore {
            bucket: database.db.gridfs_bucket(None),
        }),
        StorageConfig::S3(s3) => {
            AssetStoreKind::S3(S3AssetStore::new(s3).expect("Failed to initialise S3 asset store"))
        }
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
        asset_service: AssetService {
            project_repo: project_repo.clone(),
            team_repo: team_repo.clone(),
            asset_store,
        },
    });

    // Create ProjectServer instance (actor-less implementation). It owns a repo
    // handle so collaboration rooms can persist live CRDT text back to MongoDB.
    let ws_config = config.ws.clone();
    let project_server = ProjectServer::new(project_repo.clone(), ws_config.clone());

    let jwt_secret = config.jwt_secret.clone();
    let address = config.address.clone();

    let factory = move || {
        let cors = config.cors();

        App::new()
            .wrap(cors)
            .app_data(data.clone())
            .app_data(web::Data::new(project_server.clone()))
            .app_data(web::Data::new(ws_config.clone()))
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
