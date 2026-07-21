use actix_web::{HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::models::response::ApiResponse;

#[derive(Serialize, Deserialize, Default, utoipa::ToSchema)]
enum HealthStatus {
    #[default]
    #[serde(rename = "healthy")]
    Healthy,
}

#[derive(Serialize, Deserialize, Default, utoipa::ToSchema)]
pub struct HealthPayload {
    status: HealthStatus,
}

#[utoipa::path(
    get,
    path = "/api/health",
    tag = "health",
    responses(
        (status = 200, description = "Service is healthy", body = crate::openapi::ApiSuccess<HealthPayload>),
    )
)]
pub async fn health() -> Result<HttpResponse> {
    let response = ApiResponse::success("Service is healthy", HealthPayload::default());
    Ok(HttpResponse::Ok().json(response))
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use actix_web::{App, test, web};

    #[actix_web::test]
    async fn test_health_returns_200() {
        let app = test::init_service(App::new().route("/health", web::get().to(health))).await;

        let req = test::TestRequest::get().uri("/health").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 200);

        let body: serde_json::Value = test::read_body_json(resp).await;
        assert_eq!(body["message"], "Service is healthy");
        assert_eq!(body["payload"]["status"], "healthy");
    }
}
