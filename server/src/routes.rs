use actix_web::web;

use crate::{handler, middleware::jwt::JwtMiddleware};

/// Register every HTTP route on the given config. Shared between the real
/// server in `main.rs` and the API integration tests, so the routing table
/// (paths, methods, which scopes sit behind the JWT middleware) cannot drift
/// between what is tested and what is deployed.
pub fn configure(cfg: &mut web::ServiceConfig, jwt_secret: String) {
    cfg.route("/api/health", web::get().to(handler::health::health))
        .route("/api/register", web::post().to(handler::user::register))
        .route("/api/login", web::post().to(handler::user::login))
        .route("/api/logout", web::post().to(handler::user::logout))
        .service(
            web::scope("/api")
                .wrap(JwtMiddleware::new(jwt_secret.clone()))
                .route("/team", web::post().to(handler::team::create))
                .route("/team/projects", web::get().to(handler::team::projects))
                .route("/project", web::post().to(handler::project::create))
                .service(
                    web::scope("/project/{id}")
                        .route("", web::get().to(handler::project::find_by_id))
                        .route("", web::put().to(handler::project::update))
                        .route(
                            "/file/{file_id}",
                            web::put().to(handler::project::update_file),
                        )
                        .route("/duplicate", web::post().to(handler::project::duplicate))
                        // Binary blobs (images, fonts). The upload body is raw
                        // bytes, so lift the default 256 KiB extractor cap.
                        .app_data(web::PayloadConfig::new(16 * 1024 * 1024))
                        .route("/blobs", web::post().to(handler::blob::upload))
                        .route("/blobs/{sha}", web::get().to(handler::blob::download)),
                )
                .service(
                    web::scope("/user")
                        .route("/me", web::get().to(handler::user::me))
                        .route("/teams", web::get().to(handler::user::teams))
                        .route("/projects", web::get().to(handler::user::projects)),
                ),
        )
        .service(
            web::scope("/ws")
                .wrap(JwtMiddleware::new(jwt_secret))
                .route("/project/{id}", web::get().to(handler::ws::ws)),
        );
}
