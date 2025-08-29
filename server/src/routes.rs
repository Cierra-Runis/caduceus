use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    config::Config,
    database::Database,
    handlers::{health, user},
    middleware::jwt_middleware,
};

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub config: Config,
}

pub fn create_routes(state: AppState) -> Router {
    let protected_routes = Router::new().layer(middleware::from_fn_with_state(
        state.clone(),
        jwt_middleware,
    ));

    Router::new()
        .route("/health", get(health::health_check))
        .route("/auth/register", post(user::register))
        .route("/auth/login", post(user::login))
        .merge(protected_routes)
        .with_state(state)
}
