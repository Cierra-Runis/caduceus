//! OpenAPI document for the REST API (the WebSocket protocol under `/ws` is
//! out of scope). The spec is checked in at `docs/openapi.json`; regenerate it
//! with `cargo run --bin gen_openapi` — a test below fails when it is stale.
//!
//! The frontend consumes the checked-in spec to generate Zod schemas
//! (`pnpm codegen:api` in `app/`), so this module is the single source of
//! truth for the wire types shared by both sides.

use utoipa::{OpenApi, ToSchema};

/// Doc-only mirror of [`crate::models::response::ApiResponse`] for responses
/// that always carry a payload. `ApiResponse.payload` is an `Option` only so
/// one struct can serve both cases; on the wire a success body is always
/// `{ "message": ..., "payload": ... }`, which is what this schema states
/// (a `ToSchema` derive on the real struct would wrongly mark `payload`
/// optional and nullable). A test below pins this equivalence.
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct ApiSuccess<T> {
    pub message: String,
    pub payload: T,
}

/// Doc-only mirror of [`crate::models::response::ApiResponse`] for bodies
/// without a payload: every error response, plus payload-less successes such
/// as logout. `#[serde(skip_serializing_if)]` removes the `payload` key
/// entirely in this case, so the wire shape is just `{ "message": ... }`.
#[derive(ToSchema)]
#[allow(dead_code)]
pub struct ApiMessage {
    pub message: String,
}

#[derive(OpenApi)]
#[openapi(
    info(
        title = "caduceus API",
        description = "REST API of the caduceus collaborative Typst editor. \
                       Protected routes authenticate via a JWT, sent either as \
                       an HttpOnly `token` cookie or an `Authorization: Bearer` \
                       header.",
    ),
    paths(
        crate::handler::health::health,
        crate::handler::user::register,
        crate::handler::user::login,
        crate::handler::user::logout,
        crate::handler::user::me,
        crate::handler::user::teams,
        crate::handler::user::projects,
        crate::handler::team::create,
        crate::handler::team::projects,
        crate::handler::project::create,
        crate::handler::project::find_by_id,
        crate::handler::project::duplicate,
        crate::handler::project::update_file,
    ),
    tags(
        (name = "health", description = "Service health"),
        (name = "user", description = "Authentication and the current user"),
        (name = "team", description = "Teams"),
        (name = "project", description = "Projects and their files"),
    )
)]
pub struct ApiDoc;

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use crate::models::response::ApiResponse;

    /// Pins the assumption behind the doc-only envelopes: a success body
    /// serializes with a required `payload` key, a payload-less body drops the
    /// key entirely. If serialization of `ApiResponse` ever changes, update
    /// `ApiSuccess`/`ApiMessage` accordingly.
    #[test]
    fn doc_envelopes_match_api_response_serialization() {
        let success = serde_json::to_value(ApiResponse::success("ok", 1)).unwrap();
        assert_eq!(success, serde_json::json!({ "message": "ok", "payload": 1 }));

        let no_payload = serde_json::to_value(ApiResponse::success_no_payload("ok")).unwrap();
        assert_eq!(no_payload, serde_json::json!({ "message": "ok" }));

        let error = serde_json::to_value(ApiResponse::error("bad")).unwrap();
        assert_eq!(error, serde_json::json!({ "message": "bad" }));
    }

    /// Fails when `docs/openapi.json` is out of date with the annotations in
    /// this crate. Regenerate with `cargo run --bin gen_openapi`.
    #[test]
    fn checked_in_spec_is_up_to_date() {
        let generated = ApiDoc::openapi()
            .to_pretty_json()
            .expect("failed to serialize OpenAPI document");
        let path = concat!(env!("CARGO_MANIFEST_DIR"), "/../docs/openapi.json");
        let checked_in = std::fs::read_to_string(path)
            .expect("docs/openapi.json is missing — run `cargo run --bin gen_openapi`");
        assert_eq!(
            checked_in.trim_end(),
            generated.trim_end(),
            "docs/openapi.json is stale — run `cargo run --bin gen_openapi`"
        );
    }
}
