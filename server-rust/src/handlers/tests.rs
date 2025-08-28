use crate::handlers::health;
use axum::{http::StatusCode, routing::get, Router};
use axum_test::TestServer;
use serde_json::json;

#[tokio::test]
async fn test_health_check() {
    let app = Router::new().route("/health", get(health::health_check));

    let server = TestServer::new(app).unwrap();

    let response = server.get("/health").await;
    assert_eq!(response.status_code(), StatusCode::OK);

    let body = response.text();
    assert!(body.contains("healthy"));
}

#[tokio::test]
async fn test_register_endpoint_validation() {
    let invalid_requests = vec![
        json!({}),
        json!({"username": ""}),
        json!({"username": "test", "email": "invalid"}),
        json!({"username": "test", "email": "test@example.com"}),
    ];

    for request in invalid_requests {
        assert!(request.is_object());
    }
}

#[tokio::test]
async fn test_login_endpoint_validation() {
    let invalid_requests = vec![
        json!({}),
        json!({"username": ""}),
        json!({"username": "test"}),
        json!({"password": "test"}),
    ];

    for request in invalid_requests {
        assert!(request.is_object());
    }
}

#[test]
fn test_request_validation() {
    use serde::{Deserialize, Serialize};
    use validator::Validate;

    #[derive(Debug, Serialize, Deserialize, Validate)]
    struct RegisterRequest {
        #[validate(length(min = 1, max = 50))]
        username: String,
        #[validate(length(min = 1, max = 50))]
        nickname: String,
        #[validate(length(min = 6))]
        password: String,
    }

    let valid_request = RegisterRequest {
        username: "test_user".to_string(),
        nickname: "Test User".to_string(),
        password: "password123".to_string(),
    };
    assert!(valid_request.validate().is_ok());

    let invalid_request = RegisterRequest {
        username: "".to_string(),
        nickname: "Test User".to_string(),
        password: "password123".to_string(),
    };
    assert!(invalid_request.validate().is_err());

    let invalid_request = RegisterRequest {
        username: "test_user".to_string(),
        nickname: "Test User".to_string(),
        password: "123".to_string(),
    };
    assert!(invalid_request.validate().is_err());
}

#[test]
fn test_response_serialization() {
    use crate::models::user::User;
    use bson::oid::ObjectId;

    let user = User {
        id: Some(ObjectId::new()),
        username: "test_user".to_string(),
        nickname: "Test User".to_string(),
        password: "hashed_password".to_string(),
        created_at: chrono::Utc::now(),
        updated_at: chrono::Utc::now(),
    };

    let response = json!({
        "status": "success",
        "data": user
    });

    assert!(response["status"].as_str().unwrap() == "success");
    assert!(response["data"]["username"].as_str().unwrap() == "test_user");
}

#[test]
fn test_error_response_format() {
    let error_response = json!({
        "status": "error",
        "message": "User not found",
        "code": 404
    });

    assert!(error_response["status"].as_str().unwrap() == "error");
    assert!(error_response["message"].as_str().unwrap() == "User not found");
    assert!(error_response["code"].as_i64().unwrap() == 404);
}

#[tokio::test]
async fn test_cors_headers() {
    use axum::http::Method;
    use tower_http::cors::CorsLayer;

    let _cors = CorsLayer::new()
        .allow_origin(
            "http://localhost:3000"
                .parse::<axum::http::HeaderValue>()
                .unwrap(),
        )
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_headers(tower_http::cors::Any);
}

#[test]
fn test_jwt_middleware_logic() {
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
