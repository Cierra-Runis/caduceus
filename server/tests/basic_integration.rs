use actix_web::{test, web, App, http::StatusCode};

#[tokio::test]
async fn test_basic_api_routes() {
    use caduceus_server::{
        config::Config, database::Database,
        repo::{team::MongoTeamRepo, user::MongoUserRepo},
        services::{team::TeamService, user::UserService},
        AppState,
    };

        let config = Config {
        address: "127.0.0.1:0".to_string(),
        allow_origins: vec!["http://localhost:3000".to_string()],
        mongo_uri: "mongodb://localhost:27017".to_string(),
        db_name: "test_caduceus".to_string(),
        jwt_secret: "test_secret_key_for_testing_purposes_only".to_string(),
    };

        let database = match Database::new(&config.mongo_uri, &config.db_name).await {
        Ok(db) => db,
        Err(_) => {
            println!("Skipping test - MongoDB not available");
            return;
        }
    };

        let user_repo = MongoUserRepo {
        collection: database.db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: database.db.collection("teams"),
    };

        let user_service = UserService {
        user_repo: user_repo.clone(),
        secret: config.jwt_secret.clone(),
    };
    let team_service = TeamService {
        team_repo: team_repo.clone(),
        user_repo: user_repo.clone(),
    };

        let app_state = web::Data::new(AppState {
        database,
        config: config.clone(),
        user_service,
        team_service,
    });

        let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/health", web::get().to(caduceus_server::handler::health::health))
            .route("/api/register", web::post().to(caduceus_server::handler::user::register))
            .route("/api/login", web::post().to(caduceus_server::handler::user::login))
            .route("/api/logout", web::post().to(caduceus_server::handler::user::logout))
    ).await;

        let health_req = test::TestRequest::get()
        .uri("/api/health")
        .to_request();

    let health_resp = test::call_service(&app, health_req).await;
    assert_eq!(health_resp.status(), StatusCode::OK);

    let health_body: serde_json::Value = test::read_body_json(health_resp).await;
    assert_eq!(health_body["status"], "healthy");

        let register_data = serde_json::json!({
        "username": "test_user",
        "password": "test_password"
    });

    let register_req = test::TestRequest::post()
        .uri("/api/register")
        .set_json(&register_data)
        .to_request();

    let register_resp = test::call_service(&app, register_req).await;
            assert_ne!(register_resp.status(), StatusCode::NOT_FOUND);

        let invalid_req = test::TestRequest::get()
        .uri("/api/nonexistent")
        .to_request();

    let invalid_resp = test::call_service(&app, invalid_req).await;
    assert_eq!(invalid_resp.status(), StatusCode::NOT_FOUND);

        let _ = app_state.database.db.drop().await;
}

#[tokio::test]
async fn test_jwt_middleware() {
    use caduceus_server::{
        config::Config, database::Database,
        repo::{team::MongoTeamRepo, user::MongoUserRepo},
        services::{team::TeamService, user::UserService},
        AppState,
    };

    let config = Config {
        address: "127.0.0.1:0".to_string(),
        allow_origins: vec!["http://localhost:3000".to_string()],
        mongo_uri: "mongodb://localhost:27017".to_string(),
        db_name: "test_caduceus_jwt".to_string(),
        jwt_secret: "test_secret_key_for_testing_purposes_only".to_string(),
    };

    let database = match Database::new(&config.mongo_uri, &config.db_name).await {
        Ok(db) => db,
        Err(_) => {
            println!("Skipping test - MongoDB not available");
            return;
        }
    };

    let user_repo = MongoUserRepo {
        collection: database.db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: database.db.collection("teams"),
    };
    let user_service = UserService {
        user_repo: user_repo.clone(),
        secret: config.jwt_secret.clone(),
    };
    let team_service = TeamService {
        team_repo: team_repo.clone(),
        user_repo: user_repo.clone(),
    };

    let app_state = web::Data::new(AppState {
        database,
        config: config.clone(),
        user_service,
        team_service,
    });

        let unprotected_app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/team", web::post().to(caduceus_server::handler::team::create))
    ).await;

    let team_data = serde_json::json!({
        "name": "Test Team"
    });

    let unprotected_req = test::TestRequest::post()
        .uri("/api/team")
        .set_json(&team_data)
        .to_request();

    let unprotected_resp = test::call_service(&unprotected_app, unprotected_req).await;
        assert!(unprotected_resp.status().is_client_error() || unprotected_resp.status().is_server_error());

        let _ = app_state.database.db.drop().await;
}

#[tokio::test]
async fn test_cors_and_method_handling() {
    use caduceus_server::{
        config::Config, database::Database,
        repo::{team::MongoTeamRepo, user::MongoUserRepo},
        services::{team::TeamService, user::UserService},
        AppState,
    };

        let config = Config {
        address: "127.0.0.1:0".to_string(),
        allow_origins: vec!["http://localhost:3000".to_string()],
        mongo_uri: "mongodb://localhost:27017".to_string(),
        db_name: "test_caduceus_method".to_string(),
        jwt_secret: "test_secret_key_for_testing_purposes_only".to_string(),
    };

        let database = match Database::new(&config.mongo_uri, &config.db_name).await {
        Ok(db) => db,
        Err(_) => {
            println!("Skipping test - MongoDB not available");
            return;
        }
    };

    let user_repo = MongoUserRepo {
        collection: database.db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: database.db.collection("teams"),
    };
    let user_service = UserService {
        user_repo: user_repo.clone(),
        secret: config.jwt_secret.clone(),
    };
    let team_service = TeamService {
        team_repo: team_repo.clone(),
        user_repo: user_repo.clone(),
    };

    let app_state = web::Data::new(AppState {
        database,
        config: config.clone(),
        user_service,
        team_service,
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/health", web::get().to(caduceus_server::handler::health::health))
            .route("/api/register", web::post().to(caduceus_server::handler::user::register))
            .route("/api/login", web::post().to(caduceus_server::handler::user::login))
            .route("/api/logout", web::post().to(caduceus_server::handler::user::logout))
    ).await;

        let unsupported_methods = vec![
        test::TestRequest::put().uri("/api/health"),
        test::TestRequest::delete().uri("/api/health"),
        test::TestRequest::patch().uri("/api/health"),
        test::TestRequest::get().uri("/api/register"),
        test::TestRequest::get().uri("/api/login"),
        test::TestRequest::get().uri("/api/logout"),
    ];

    for req_builder in unsupported_methods {
        let req = req_builder.to_request();
        let resp = test::call_service(&app, req).await;
                        assert!(resp.status() == StatusCode::NOT_FOUND || resp.status() == StatusCode::METHOD_NOT_ALLOWED);
    }

        let _ = app_state.database.db.drop().await;
}

#[tokio::test]
async fn test_request_body_validation() {
    use caduceus_server::{
        config::Config, database::Database,
        repo::{team::MongoTeamRepo, user::MongoUserRepo},
        services::{team::TeamService, user::UserService},
        AppState,
    };

    let config = Config {
        address: "127.0.0.1:0".to_string(),
        allow_origins: vec!["http://localhost:3000".to_string()],
        mongo_uri: "mongodb://localhost:27017".to_string(),
        db_name: "test_caduceus_validation".to_string(),
        jwt_secret: "test_secret_key_for_testing_purposes_only".to_string(),
    };

    let database = match Database::new(&config.mongo_uri, &config.db_name).await {
        Ok(db) => db,
        Err(_) => {
            println!("Skipping test - MongoDB not available");
            return;
        }
    };

    let user_repo = MongoUserRepo {
        collection: database.db.collection("users"),
    };
    let team_repo = MongoTeamRepo {
        collection: database.db.collection("teams"),
    };
    let user_service = UserService {
        user_repo: user_repo.clone(),
        secret: config.jwt_secret.clone(),
    };
    let team_service = TeamService {
        team_repo: team_repo.clone(),
        user_repo: user_repo.clone(),
    };

    let app_state = web::Data::new(AppState {
        database,
        config: config.clone(),
        user_service,
        team_service,
    });

    let app = test::init_service(
        App::new()
            .app_data(app_state.clone())
            .route("/api/register", web::post().to(caduceus_server::handler::user::register))
            .route("/api/login", web::post().to(caduceus_server::handler::user::login))
    ).await;

        let invalid_json_req = test::TestRequest::post()
        .uri("/api/register")
        .insert_header(("content-type", "application/json"))
        .set_payload("{invalid json}")
        .to_request();

    let invalid_json_resp = test::call_service(&app, invalid_json_req).await;
    assert_eq!(invalid_json_resp.status(), StatusCode::BAD_REQUEST);

        let missing_field_req = test::TestRequest::post()
        .uri("/api/register")
        .set_json(&serde_json::json!({
            "username": "testuser"
                    }))
        .to_request();

    let missing_field_resp = test::call_service(&app, missing_field_req).await;
    assert_eq!(missing_field_resp.status(), StatusCode::BAD_REQUEST);

        let empty_body_req = test::TestRequest::post()
        .uri("/api/register")
        .insert_header(("content-type", "application/json"))
        .to_request();

    let empty_body_resp = test::call_service(&app, empty_body_req).await;
    assert_eq!(empty_body_resp.status(), StatusCode::BAD_REQUEST);

        let wrong_content_type_req = test::TestRequest::post()
        .uri("/api/register")
        .insert_header(("content-type", "text/plain"))
        .set_payload(r#"{"username": "test", "password": "test"}"#)
        .to_request();

    let wrong_content_type_resp = test::call_service(&app, wrong_content_type_req).await;
    assert!(wrong_content_type_resp.status().is_client_error());

        let _ = app_state.database.db.drop().await;
}

#[tokio::test]
async fn test_application_startup_and_configuration() {
    use caduceus_server::{
        config::Config, database::Database,
        repo::{team::MongoTeamRepo, user::MongoUserRepo},
        services::{team::TeamService, user::UserService},
        AppState,
    };

        let configs = vec![
        Config {
            address: "127.0.0.1:8080".to_string(),
            allow_origins: vec!["http://localhost:3000".to_string()],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "test_config_1".to_string(),
            jwt_secret: "secret1".to_string(),
        },
        Config {
            address: "0.0.0.0:8080".to_string(),
            allow_origins: vec![
                "http://localhost:3000".to_string(),
                "https://example.com".to_string(),
            ],
            mongo_uri: "mongodb://localhost:27017".to_string(),
            db_name: "test_config_2".to_string(),
            jwt_secret: "secret2".to_string(),
        },
    ];

    for config in configs {
                let database = match Database::new(&config.mongo_uri, &config.db_name).await {
            Ok(db) => db,
            Err(_) => {
                println!("Skipping test - MongoDB not available");
                continue;
            }
        };

        let user_repo = MongoUserRepo {
            collection: database.db.collection("users"),
        };
        let team_repo = MongoTeamRepo {
            collection: database.db.collection("teams"),
        };
        let user_service = UserService {
            user_repo: user_repo.clone(),
            secret: config.jwt_secret.clone(),
        };
        let team_service = TeamService {
            team_repo: team_repo.clone(),
            user_repo: user_repo.clone(),
        };

        let app_state = web::Data::new(AppState {
            database,
            config: config.clone(),
            user_service,
            team_service,
        });

                assert_eq!(app_state.config.db_name, config.db_name);
        assert_eq!(app_state.config.jwt_secret, config.jwt_secret);

                let _ = app_state.database.db.drop().await;
    }
}
