//! Writes the OpenAPI spec to `docs/openapi.json` (repo root). Run from
//! anywhere: `cargo run --bin gen_openapi`. Purely static — no server, no DB.

use server::openapi::ApiDoc;
use std::{fs, path::Path};
use utoipa::OpenApi;

fn main() {
    let json = ApiDoc::openapi()
        .to_pretty_json()
        .expect("failed to serialize OpenAPI document");
    let path = Path::new(env!("CARGO_MANIFEST_DIR")).join("../docs/openapi.json");
    fs::write(&path, json + "\n").expect("failed to write docs/openapi.json");
    println!("wrote {}", path.display());
}
