use axum::Json;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Response<T> {
    pub data: Option<T>,
    pub message: String,
}

pub type JsonResponse<T> = Json<Response<T>>;
