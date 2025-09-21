#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod config;
pub mod database;
pub mod handler;
pub mod middleware;
pub mod models;
pub mod repo;
pub mod services;

use crate::{
    repo::{project::MongoProjectRepo, team::MongoTeamRepo, user::MongoUserRepo},
    services::{project::ProjectService, team::TeamService, user::UserService},
};

pub struct AppState {
    pub user_service: UserService<MongoUserRepo, MongoTeamRepo, MongoProjectRepo>,
    pub team_service: TeamService<MongoTeamRepo, MongoUserRepo>,
    pub project_service: ProjectService<MongoProjectRepo, MongoUserRepo, MongoTeamRepo>,
}
