use actix_web::web;

use crate::{handler, middleware::jwt::JwtMiddleware};

/// Maximum accepted size for a single binary-asset upload (25 MiB) — comfortably
/// covers images and fonts while bounding memory per request.
const ASSET_UPLOAD_LIMIT: usize = 25 * 1024 * 1024;

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
                        // Binary-asset upload accepts a raw body; raise the
                        // extractor's payload cap (default 256 KiB) so images and
                        // fonts fit. Scoped to this resource so other endpoints
                        // keep the tight default.
                        .service(
                            web::resource("/asset")
                                .app_data(web::PayloadConfig::new(ASSET_UPLOAD_LIMIT))
                                .route(web::post().to(handler::asset::upload_asset)),
                        )
                        .route(
                            "/asset/{file_id}",
                            web::get().to(handler::asset::get_asset),
                        )
                        .route(
                            "/asset/{file_id}",
                            web::delete().to(handler::asset::delete_asset),
                        )
                        .route("/duplicate", web::post().to(handler::project::duplicate)),
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
