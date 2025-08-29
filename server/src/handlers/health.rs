use axum::Json;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use crate::models::response::{JsonResponse, Response};

#[derive(Serialize, Deserialize, Debug, Default)]
pub enum HealthStatus {
    #[default]
    #[serde(rename = "healthy")]
    Healthy,
    #[serde(rename = "unhealthy")]
    Unhealthy,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct HealthPayload {
    status: HealthStatus,
    timestamp: DateTime<Utc>,
}

impl Default for HealthPayload {
    fn default() -> Self {
        HealthPayload {
            status: HealthStatus::default(),
            timestamp: Utc::now(),
        }
    }
}

pub async fn health_check() -> JsonResponse<HealthPayload> {
    Json(Response {
        data: Some(HealthPayload::default()),
        message: "Service is healthy".to_string(),
    })
}
