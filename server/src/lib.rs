#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod config;
pub mod database;
pub mod font;
pub mod handler;
pub mod middleware;
pub mod models;
pub mod repo;
pub mod routes;
pub mod services;
pub mod storage;

use std::sync::Arc;

use crate::{
    repo::{project::MongoProjectRepo, team::MongoTeamRepo, user::MongoUserRepo},
    services::{project::ProjectService, team::TeamService, user::UserService},
    storage::ObjectStore,
};

pub struct AppState {
    pub user_service: UserService<MongoUserRepo, MongoTeamRepo, MongoProjectRepo>,
    pub team_service: TeamService<MongoTeamRepo, MongoUserRepo, MongoProjectRepo>,
    pub project_service: ProjectService<MongoProjectRepo, MongoUserRepo, MongoTeamRepo>,
    /// Backing store for binary assets (see [`storage`]). A trait object so the
    /// concrete backend (MinIO in production, in-memory in tests) is chosen at
    /// wiring time, not baked into every handler signature.
    pub object_store: Arc<dyn ObjectStore>,
}
