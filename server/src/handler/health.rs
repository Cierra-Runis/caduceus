use actix_web::{HttpResponse, Result};
use serde::{Deserialize, Serialize};

use crate::models::response::ApiResponse;

#[derive(Serialize, Deserialize, Default)]
enum HealthStatus {
    #[default]
    #[serde(rename = "healthy")]
    Healthy,
}

#[derive(Serialize, Deserialize, Default)]
pub struct HealthPayload {
    status: HealthStatus,
}

pub async fn health() -> Result<HttpResponse> {
    let response = ApiResponse::success("Service is healthy", HealthPayload::default());
    Ok(HttpResponse::Ok().json(response))
}
