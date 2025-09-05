#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod config;
pub mod database;
pub mod handler;
pub mod middleware;
pub mod models;
pub mod repo;
pub mod services;

use crate::{
    repo::{team::MongoTeamRepo, user::MongoUserRepo},
    services::{team::TeamService, user::UserService},
};
use database::Database;

pub struct AppState {
    pub database: Database,
    pub user_service: UserService<MongoUserRepo, MongoTeamRepo>,
    pub team_service: TeamService<MongoTeamRepo, MongoUserRepo>,
}
