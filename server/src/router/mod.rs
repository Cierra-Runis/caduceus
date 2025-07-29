use axum::Router;
use mongodb::{Collection, Database};
use tower_http::trace::TraceLayer;

use crate::model::user::User;

pub fn app(database: Database) -> Router {
    let collection: Collection<User> = database.collection("user");

    Router::new()
        .layer(TraceLayer::new_for_http())
        .with_state(collection)
}
