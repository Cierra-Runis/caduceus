//! End-to-end HTTP tests: real router (`server::routes::configure`), real JWT
//! middleware, real Mongo-backed services against the `config/test.yaml`
//! database. Requires a running MongoDB (provisioned in CI); locally these
//! fail with connection-refused, same as the repo-layer integration tests.

use actix_web::{
    App,
    body::MessageBody,
    cookie::Cookie,
    dev::{Service, ServiceResponse},
    test, web,
};
use bson::oid::ObjectId;
use server::{
    AppState,
    config::Config,
    repo::{
        asset::{AssetStoreKind, GridFsAssetStore},
        project::MongoProjectRepo,
        team::MongoTeamRepo,
        user::MongoUserRepo,
    },
    routes,
    services::{
        asset::AssetService, project::ProjectService, team::TeamService, user::UserService,
    },
};

async fn test_app() -> (
    impl Service<actix_http::Request, Response = ServiceResponse<impl MessageBody>, Error = actix_web::Error>,
    Config,
) {
    let config = Config::load("config/test.yaml").unwrap();
    let db = mongodb::Client::with_uri_str(&config.mongo_uri)
        .await
        .unwrap()
        .database(&config.db_name);

    let user_repo = MongoUserRepo {
        collection: db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: db.collection("teams"),
    };
    let project_repo = MongoProjectRepo {
        collection: db.collection("projects"),
    };

    let data = web::Data::new(AppState {
        user_service: UserService {
            user_repo: user_repo.clone(),
            team_repo: team_repo.clone(),
            project_repo: project_repo.clone(),
            secret: config.jwt_secret.clone(),
        },
        team_service: TeamService {
            team_repo: team_repo.clone(),
            user_repo: user_repo.clone(),
            project_repo: project_repo.clone(),
        },
        project_service: ProjectService {
            project_repo: project_repo.clone(),
            user_repo,
            team_repo: team_repo.clone(),
        },
        asset_service: AssetService {
            project_repo,
            team_repo,
            asset_store: AssetStoreKind::GridFs(GridFsAssetStore {
                bucket: db.gridfs_bucket(None),
            }),
        },
    });

    let jwt_secret = config.jwt_secret.clone();
    let app = test::init_service(
        App::new()
            .app_data(data)
            .configure(move |cfg| routes::configure(cfg, jwt_secret.clone())),
    )
    .await;
    (app, config)
}

fn unique_username() -> String {
    format!("it_user_{}", ObjectId::new().to_hex())
}

/// Register a fresh user and return `(token cookie, user id, username)`.
async fn register_user<S, B>(app: &S) -> (Cookie<'static>, String, String)
where
    S: Service<actix_http::Request, Response = ServiceResponse<B>, Error = actix_web::Error>,
    B: MessageBody,
{
    let username = unique_username();
    let req = test::TestRequest::post()
        .uri("/api/register")
        .set_json(serde_json::json!({ "username": username, "password": "password123" }))
        .to_request();
    let resp = test::call_service(app, req).await;
    assert_eq!(resp.status(), 200, "register must succeed");

    let cookie = resp
        .response()
        .cookies()
        .find(|c| c.name() == "token")
        .expect("register must set the token cookie")
        .into_owned();
    assert!(!cookie.value().is_empty());

    let body: serde_json::Value = test::read_body_json(resp).await;
    let user_id = body["payload"]["user"]["id"].as_str().unwrap().to_string();
    (cookie, user_id, username)
}

#[actix_web::test]
async fn test_register_login_me_flow() {
    let (app, _config) = test_app().await;
    let (cookie, user_id, username) = register_user(&app).await;

    // Authenticated /user/me via the cookie the server itself issued.
    let req = test::TestRequest::get()
        .uri("/api/user/me")
        .cookie(cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["payload"]["username"], username.as_str());
    assert_eq!(body["payload"]["id"], user_id.as_str());

    // Same username again → 409.
    let req = test::TestRequest::post()
        .uri("/api/register")
        .set_json(serde_json::json!({ "username": username, "password": "password123" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 409);

    // Wrong password → 401.
    let req = test::TestRequest::post()
        .uri("/api/login")
        .set_json(serde_json::json!({ "username": username, "password": "wrong" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);

    // Correct login → 200 and a fresh token cookie.
    let req = test::TestRequest::post()
        .uri("/api/login")
        .set_json(serde_json::json!({ "username": username, "password": "password123" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    assert!(resp.response().cookies().any(|c| c.name() == "token"));
}

#[actix_web::test]
async fn test_protected_route_requires_token() {
    let (app, _config) = test_app().await;
    let req = test::TestRequest::get().uri("/api/user/me").to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 401);
}

#[actix_web::test]
async fn test_project_crud_flow() {
    let (app, _config) = test_app().await;
    let (cookie, user_id, _) = register_user(&app).await;

    // Create a user-owned project.
    let req = test::TestRequest::post()
        .uri("/api/project")
        .cookie(cookie.clone())
        .set_json(serde_json::json!({
            "owner_id": user_id,
            "owner_type": "user",
            "name": "it project",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let project_id = body["payload"]["id"].as_str().unwrap().to_string();

    // It shows up in the user's project list.
    let req = test::TestRequest::get()
        .uri("/api/user/projects")
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["payload"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["id"] == project_id.as_str()),
        "created project must appear in /user/projects"
    );

    // Detail payload carries the seeded entry file with inlined text content.
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let entry = body["payload"]["entry"].as_str().unwrap().to_string();
    let files = body["payload"]["files"].as_array().unwrap();
    let entry_file = files.iter().find(|f| f["id"] == entry.as_str()).unwrap();
    assert_eq!(entry_file["content"]["kind"], "text");
    let initial_version = entry_file["version"].as_i64().unwrap();

    // Update the entry file: version bumps.
    let req = test::TestRequest::put()
        .uri(&format!("/api/project/{project_id}/file/{entry}"))
        .cookie(cookie.clone())
        .set_json(serde_json::json!({ "text": "= Updated" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["payload"]["version"], initial_version + 1);

    // Nonexistent file id in a real project → 404 (regression guard for the
    // array_filters bug fixed in repo::project).
    let req = test::TestRequest::put()
        .uri(&format!(
            "/api/project/{project_id}/file/{}",
            ObjectId::new().to_hex()
        ))
        .cookie(cookie.clone())
        .set_json(serde_json::json!({ "text": "x" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // Duplicate: fresh id, derived name.
    let req = test::TestRequest::post()
        .uri(&format!("/api/project/{project_id}/duplicate"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_ne!(body["payload"]["id"], project_id.as_str());
    assert_eq!(body["payload"]["name"], "it project copy");

    // Rename via PUT: name changes, ownership stays.
    let req = test::TestRequest::put()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(cookie.clone())
        .set_json(serde_json::json!({
            "owner_id": user_id,
            "owner_type": "user",
            "name": "it project renamed",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["payload"]["name"], "it project renamed");
    assert_eq!(body["payload"]["owner_id"], user_id.as_str());

    // A stranger cannot read the project.
    let (other_cookie, other_user_id, _) = register_user(&app).await;
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(other_cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Nor update it — even to claim it for themselves.
    let req = test::TestRequest::put()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(other_cookie)
        .set_json(serde_json::json!({
            "owner_id": other_user_id,
            "owner_type": "user",
            "name": "stolen",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Unknown and malformed ids → 404.
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{}", ObjectId::new().to_hex()))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
    let req = test::TestRequest::get()
        .uri("/api/project/not-an-object-id")
        .cookie(cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_asset_upload_download_flow() {
    let (app, _config) = test_app().await;
    let (cookie, user_id, _) = register_user(&app).await;

    // Create a project to attach the asset to.
    let req = test::TestRequest::post()
        .uri("/api/project")
        .cookie(cookie.clone())
        .set_json(serde_json::json!({
            "owner_id": user_id,
            "owner_type": "user",
            "name": "asset project",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let project_id = body["payload"]["id"].as_str().unwrap().to_string();

    // Upload a binary asset (a tiny PNG-ish byte blob) with a content type.
    let bytes = vec![0x89u8, 0x50, 0x4E, 0x47, 0x0D, 0x0A, 0x1A, 0x0A];
    let req = test::TestRequest::post()
        .uri(&format!("/api/project/{project_id}/asset?path=logo.png"))
        .cookie(cookie.clone())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "image/png"))
        .set_payload(bytes.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["payload"]["path"], "logo.png");
    assert_eq!(body["payload"]["content"]["kind"], "binary");
    assert_eq!(body["payload"]["size"].as_i64().unwrap(), bytes.len() as i64);
    let file_id = body["payload"]["id"].as_str().unwrap().to_string();

    // The detail payload now lists the binary file as a storage-key reference.
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let asset_file = body["payload"]["files"]
        .as_array()
        .unwrap()
        .iter()
        .find(|f| f["id"] == file_id.as_str())
        .unwrap();
    assert_eq!(asset_file["content"]["kind"], "binary");
    assert!(
        asset_file["content"]["storageKey"]
            .as_str()
            .is_some_and(|k| !k.is_empty())
    );

    // Fetch the bytes back with the stored content type.
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}/asset/{file_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    assert_eq!(
        resp.headers()
            .get(actix_web::http::header::CONTENT_TYPE)
            .map(|v| v.to_str().unwrap()),
        Some("image/png")
    );
    let returned = test::read_body(resp).await;
    assert_eq!(returned.as_ref(), bytes.as_slice());

    // A stranger cannot read the asset.
    let (other_cookie, _, _) = register_user(&app).await;
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}/asset/{file_id}"))
        .cookie(other_cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Nor upload into it.
    let req = test::TestRequest::post()
        .uri(&format!("/api/project/{project_id}/asset?path=evil.png"))
        .cookie(other_cookie.clone())
        .insert_header((actix_web::http::header::CONTENT_TYPE, "image/png"))
        .set_payload(vec![1u8, 2, 3])
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Fetching the project's seeded text file *as an asset* is a 400 (not a
    // binary), and an unknown file id is a 404.
    let entry = body["payload"]["entry"].as_str().unwrap().to_string();
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}/asset/{entry}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/project/{project_id}/asset/{}",
            ObjectId::new().to_hex()
        ))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // A stranger cannot delete the asset.
    let req = test::TestRequest::delete()
        .uri(&format!("/api/project/{project_id}/asset/{file_id}"))
        .cookie(other_cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Deleting the compile entry is refused (400) so the project stays
    // compilable.
    let req = test::TestRequest::delete()
        .uri(&format!("/api/project/{project_id}/asset/{entry}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 400);

    // The owner deletes the uploaded asset.
    let req = test::TestRequest::delete()
        .uri(&format!("/api/project/{project_id}/asset/{file_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    // It is gone from the detail payload…
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["payload"]["files"]
            .as_array()
            .unwrap()
            .iter()
            .all(|f| f["id"] != file_id.as_str())
    );

    // …and fetching its bytes now 404s.
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}/asset/{file_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);

    // Deleting an already-gone asset is a 404 (no such file row).
    let req = test::TestRequest::delete()
        .uri(&format!("/api/project/{project_id}/asset/{file_id}"))
        .cookie(cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}

#[actix_web::test]
async fn test_team_flow() {
    let (app, _config) = test_app().await;
    let (cookie, user_id, _) = register_user(&app).await;

    let req = test::TestRequest::post()
        .uri("/api/team")
        .cookie(cookie.clone())
        .set_json(serde_json::json!({ "name": "it team" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    let team_id = body["payload"]["id"].as_str().unwrap().to_string();
    assert_eq!(body["payload"]["creator_id"], user_id.as_str());

    // Creator is a member; the team lists under /user/teams.
    let req = test::TestRequest::get()
        .uri("/api/user/teams")
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["payload"]
            .as_array()
            .unwrap()
            .iter()
            .any(|t| t["id"] == team_id.as_str())
    );

    // Team-owned project via the team query endpoint.
    let req = test::TestRequest::post()
        .uri("/api/project")
        .cookie(cookie.clone())
        .set_json(serde_json::json!({
            "owner_id": team_id,
            "owner_type": "team",
            "name": "it team project",
        }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);

    let req = test::TestRequest::get()
        .uri(&format!("/api/team/projects?id={team_id}"))
        .cookie(cookie.clone())
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert!(
        body["payload"]
            .as_array()
            .unwrap()
            .iter()
            .any(|p| p["name"] == "it team project")
    );

    // A logged-in user who is not a member of the team must not be able to
    // list its projects — knowing the team id is not enough.
    let (other_cookie, _, _) = register_user(&app).await;
    let req = test::TestRequest::get()
        .uri(&format!("/api/team/projects?id={team_id}"))
        .cookie(other_cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 403);

    // Nonexistent team → 404, even for an authenticated caller.
    let req = test::TestRequest::get()
        .uri(&format!(
            "/api/team/projects?id={}",
            ObjectId::new().to_hex()
        ))
        .cookie(cookie)
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 404);
}
