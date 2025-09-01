use actix_web::{HttpResponse, Result};
use serde::{Deserialize, Serialize};

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
    Ok(HttpResponse::Ok().json(HealthPayload::default()))
}
