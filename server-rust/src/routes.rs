use axum::{
    middleware,
    routing::{get, post},
    Router,
};

use crate::{
    config::Config,
    database::Database,
    handlers::{health, project, user},
    middleware::jwt_middleware,
};

#[derive(Clone)]
pub struct AppState {
    pub database: Database,
    pub config: Config,
}

pub fn create_routes(state: AppState) -> Router {
    let protected_routes = Router::new()
        .route("/users/me", get(user::get_current_user))
        .route("/projects", post(project::create_project))
        .layer(middleware::from_fn_with_state(
            state.clone(),
            jwt_middleware,
        ));

    Router::new()
        // Health check
        .route("/health", get(health::health_check))
        // Authentication routes (no JWT required)
        .route("/auth/register", post(user::register))
        .route("/auth/login", post(user::login))
        // Merge protected routes
        .merge(protected_routes)
        .with_state(state)
}
