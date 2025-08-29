use axum::{
    extract::{FromRequest, Request, State},
    http::header::AUTHORIZATION,
    middleware::Next,
    response::Response,
    Json,
};
use jsonwebtoken::{decode, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use validator::Validate;

use crate::{error::AppError, routes::AppState};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Claims {
    pub sub: String,
    pub username: String,
    pub exp: usize,
}

pub async fn jwt_middleware(
    State(state): State<AppState>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get(AUTHORIZATION)
        .and_then(|header| header.to_str().ok());

    let token = match auth_header {
        Some(header) if header.starts_with("Bearer ") => header.trim_start_matches("Bearer "),
        _ => {
            return Err(AppError::Authentication(
                "Missing or invalid authorization header".to_string(),
            ));
        }
    };

    let validation = Validation::default();
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(state.config.jwt_secret.as_ref()),
        &validation,
    )?;

    request.extensions_mut().insert(token_data.claims);

    Ok(next.run(request).await)
}

pub struct ValidatedJson<T>(pub T);

impl<T, S> FromRequest<S> for ValidatedJson<T>
where
    T: for<'de> Deserialize<'de> + Validate,
    S: Send + Sync,
{
    type Rejection = AppError;

    async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
        let Json(value) = Json::<T>::from_request(req, state)
            .await
            .map_err(|e| AppError::Validation(format!("Invalid Json: {}", e)))?;

        value
            .validate()
            .map_err(|e| AppError::Validation(format!("Validation error: {}", e)))?;

        Ok(ValidatedJson(value))
    }
}
