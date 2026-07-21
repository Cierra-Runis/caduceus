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
    repo::{project::MongoProjectRepo, team::MongoTeamRepo, user::MongoUserRepo},
    routes,
    services::{project::ProjectService, team::TeamService, user::UserService},
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
            project_repo,
            user_repo,
            team_repo,
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

    // Update the entry file: version bumps.
    let req = test::TestRequest::put()
        .uri(&format!("/api/project/{project_id}/file/{entry}"))
        .cookie(cookie.clone())
        .set_json(serde_json::json!({ "text": "= Updated" }))
        .to_request();
    let resp = test::call_service(&app, req).await;
    assert_eq!(resp.status(), 200);
    let body: serde_json::Value = test::read_body_json(resp).await;
    assert_eq!(body["payload"]["version"], 2);

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

    // A stranger cannot read the project.
    let (other_cookie, _, _) = register_user(&app).await;
    let req = test::TestRequest::get()
        .uri(&format!("/api/project/{project_id}"))
        .cookie(other_cookie)
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
        .cookie(cookie)
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
}
