use axum::http::StatusCode;
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_app_creation() {
    use axum::{routing::get, Router};

    async fn health() -> &'static str {
        "OK"
    }

    let app = Router::new().route("/health", get(health));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/health").await;

    assert_eq!(response.status_code(), StatusCode::OK);
    assert_eq!(response.text(), "OK");
}

#[tokio::test]
async fn test_cors_configuration() {
    use axum::http::Method;
    use axum::{routing::get, Router};
    use tower_http::cors::CorsLayer;

    async fn test_endpoint() -> &'static str {
        "test"
    }

    let cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);

    let app = Router::new().route("/test", get(test_endpoint)).layer(cors);

    let server = TestServer::new(app).unwrap();
    let response = server.get("/test").await;

    assert_eq!(response.status_code(), StatusCode::OK);
}

#[test]
fn test_validation_helpers() {
    use regex::Regex;

    let username_regex = Regex::new(r"^[a-zA-Z0-9_]{1,50}$").unwrap();
    assert!(username_regex.is_match("valid_username"));
    assert!(!username_regex.is_match("invalid@username"));
    assert!(!username_regex.is_match(""));
    assert!(!username_regex.is_match(&"a".repeat(51)));

    fn validate_password(password: &str) -> bool {
        password.len() >= 6
    }

    assert!(validate_password("password123"));
    assert!(!validate_password("123"));
}

#[tokio::test]
async fn test_json_serialization_in_handlers() {
    use axum::{routing::get, Json, Router};
    use serde_json::Value;

    async fn json_handler() -> Json<Value> {
        Json(json!({
            "status": "success",
            "data": {
                "message": "Hello, World!"
            }
        }))
    }

    let app = Router::new().route("/json", get(json_handler));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/json").await;

    assert_eq!(response.status_code(), StatusCode::OK);

    let json_response: Value = response.json();
    assert_eq!(json_response["status"], "success");
    assert_eq!(json_response["data"]["message"], "Hello, World!");
}

#[test]
fn test_password_hashing() {
    let password = "test_password";
    let hashed = bcrypt::hash(password, bcrypt::DEFAULT_COST).unwrap();

    assert_ne!(password, hashed);
    assert!(bcrypt::verify(password, &hashed).unwrap());
    assert!(!bcrypt::verify("wrong_password", &hashed).unwrap());
}

#[test]
fn test_jwt_token_operations() {
    use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize, Deserialize)]
    struct Claims {
        sub: String,
        exp: usize,
    }

    let secret = "test_secret";
    let user_id = "test_user_id";

    let exp = chrono::Utc::now()
        .checked_add_signed(chrono::Duration::hours(24))
        .expect("valid timestamp")
        .timestamp() as usize;

    let claims = Claims {
        sub: user_id.to_string(),
        exp,
    };

    let token = encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .unwrap();

    let token_data = decode::<Claims>(
        &token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    assert!(token_data.is_ok());
    assert_eq!(token_data.unwrap().claims.sub, user_id);

    let invalid_token = "invalid.token.here";
    let invalid_result = decode::<Claims>(
        invalid_token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    );

    assert!(invalid_result.is_err());
}

#[tokio::test]
async fn test_error_response_format() {
    use axum::{
        response::{IntoResponse, Response},
        routing::get,
        Json, Router,
    };
    use serde_json::Value;

    async fn error_handler() -> Response {
        let error_response = json!({
            "success": false,
            "message": "User not found",
            "code": 404
        });

        (StatusCode::NOT_FOUND, Json(error_response)).into_response()
    }

    let app = Router::new().route("/error", get(error_handler));

    let server = TestServer::new(app).unwrap();
    let response = server.get("/error").await;

    assert_eq!(response.status_code(), StatusCode::NOT_FOUND);

    let json_response: Value = response.json();
    assert_eq!(json_response["success"], false);
    assert_eq!(json_response["message"], "User not found");
    assert_eq!(json_response["code"], 404);
}
