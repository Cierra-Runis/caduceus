use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    Error, FromRequest, HttpMessage, HttpRequest, HttpResponse, Result,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde_json;
use std::{
    future::{ready, Ready as StdReady},
    rc::Rc,
};

use crate::models::user::UserClaims;

impl FromRequest for UserClaims {
    type Error = Error;
    type Future = StdReady<Result<Self, Self::Error>>;

    fn from_request(req: &HttpRequest, _: &mut actix_web::dev::Payload) -> Self::Future {
        let req = req.clone();
        ready(get_user_from_request(&req))
    }
}

fn get_user_from_request(req: &HttpRequest) -> Result<UserClaims, Error> {
    req.extensions()
        .get::<UserClaims>()
        .cloned()
        .ok_or_else(|| actix_web::error::ErrorUnauthorized("JWT token required"))
}

pub fn extract_token_from_request(req: &ServiceRequest) -> Option<String> {
    let auth_header = req.headers().get(header::AUTHORIZATION)?;
    let auth_str = auth_header.to_str().ok()?;

    if let Some(token) = auth_str.strip_prefix("Bearer ") {
        return Some(token.to_string());
    }

    Some(req.cookie("token")?.value().to_string())
}

pub fn verify_jwt(token: &str, secret: &str) -> Result<UserClaims, jsonwebtoken::errors::Error> {
    let validation = Validation::new(Algorithm::HS512);

    let token_data = decode::<UserClaims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &validation,
    )?;

    Ok(token_data.claims)
}

pub struct JwtMiddleware {
    jwt_secret: String,
}

impl JwtMiddleware {
    pub fn new(jwt_secret: String) -> Self {
        Self { jwt_secret }
    }
}

impl<S> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareService {
            service: Rc::new(service),
            jwt_secret: self.jwt_secret.clone(),
        })
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
    jwt_secret: String,
}

impl<S> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse, Error = Error> + 'static,
    S::Future: 'static,
{
    type Response = ServiceResponse;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let jwt_secret = self.jwt_secret.clone();

        Box::pin(async move {
            let token = match extract_token_from_request(&req) {
                Some(token) => token,
                None => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({"message": "JWT token required"}));
                    return Ok(req.into_response(response));
                }
            };

            let claims = match verify_jwt(&token, &jwt_secret) {
                Ok(claims) => claims,
                Err(_) => {
                    let response = HttpResponse::Unauthorized()
                        .json(serde_json::json!({"message": "Invalid JWT token"}));
                    return Ok(req.into_response(response));
                }
            };

            req.extensions_mut().insert(claims);

            service.call(req).await
        })
    }
}

#[cfg(test)]
#[cfg_attr(coverage_nightly, coverage(off))]
mod tests {
    use super::*;
    use actix_web::{test, web, App, HttpResponse};

    #[actix_web::test]
    async fn test_jwt_middleware_no_token() {
        async fn protected_handler() -> HttpResponse {
            HttpResponse::Ok().finish()
        }

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware::new("test_secret".to_string()))
                .route("/protected", web::get().to(protected_handler)),
        )
        .await;

        let req = test::TestRequest::get().uri("/protected").to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
        assert_eq!(
            test::read_body(resp).await,
            serde_json::to_string(&serde_json::json!({"message": "JWT token required"}))
                .unwrap()
                .as_bytes()
        );
    }

    #[actix_web::test]
    async fn test_jwt_middleware_invalid_token() {
        async fn protected_handler() -> HttpResponse {
            HttpResponse::Ok().finish()
        }

        let app = test::init_service(
            App::new()
                .wrap(JwtMiddleware::new("test_secret".to_string()))
                .route("/protected", web::get().to(protected_handler)),
        )
        .await;

        let req = test::TestRequest::get()
            .uri("/protected")
            .insert_header((header::AUTHORIZATION, "Bearer invalid_token"))
            .to_request();
        let resp = test::call_service(&app, req).await;
        assert_eq!(resp.status(), 401);
        assert_eq!(
            test::read_body(resp).await,
            serde_json::to_string(&serde_json::json!({"message": "Invalid JWT token"}))
                .unwrap()
                .as_bytes()
        );
    }
}
