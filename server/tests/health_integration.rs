use actix_web::{test, web, App};
use caduceus_server::handler;

mod common;
use common::HealthResponse;

#[tokio::test]
async fn test_health_endpoint_returns_healthy_status() {
    let app =
        test::init_service(App::new().route("/api/health", web::get().to(handler::health::health)))
            .await;

    let req = test::TestRequest::get().uri("/api/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    let body: HealthResponse = test::read_body_json(resp).await;
    assert_eq!(body.status, "healthy");
}

#[tokio::test]
async fn test_health_endpoint_accepts_get_only() {
    let app =
        test::init_service(App::new().route("/api/health", web::get().to(handler::health::health)))
            .await;

    let req = test::TestRequest::post().uri("/api/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status() == 404 || resp.status() == 405);

    let req = test::TestRequest::put().uri("/api/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status() == 404 || resp.status() == 405);

    let req = test::TestRequest::delete().uri("/api/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status() == 404 || resp.status() == 405);
}

#[tokio::test]
async fn test_health_endpoint_response_format() {
    let app =
        test::init_service(App::new().route("/api/health", web::get().to(handler::health::health)))
            .await;

    let req = test::TestRequest::get().uri("/api/health").to_request();

    let resp = test::call_service(&app, req).await;
    assert!(resp.status().is_success());

    assert_eq!(
        resp.headers().get("content-type").unwrap(),
        "application/json"
    );

    let body: serde_json::Value = test::read_body_json(resp).await;

    assert!(body.is_object());
    assert!(body.get("status").is_some());
    assert_eq!(body["status"], "healthy");
}
