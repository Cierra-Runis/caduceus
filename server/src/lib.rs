#![cfg_attr(coverage_nightly, feature(coverage_attribute))]

pub mod config;
pub mod database;
pub mod handler;
pub mod middleware;
pub mod models;
pub mod repo;
pub mod services;

use actix_web::web;
use crate::{
    middleware::jwt::JwtMiddleware,
    repo::{team::MongoTeamRepo, user::MongoUserRepo},
    services::{team::TeamService, user::UserService},
};
use config::Config;
use database::Database;

pub struct AppState {
    pub database: Database,
    pub config: Config,
    pub user_service: UserService<MongoUserRepo>,
    pub team_service: TeamService<MongoTeamRepo, MongoUserRepo>,
}
