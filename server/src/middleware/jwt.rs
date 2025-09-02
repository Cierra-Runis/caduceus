use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    http::header,
    web, Error, FromRequest, HttpMessage, HttpRequest, Result,
};
use futures_util::future::{ok, LocalBoxFuture, Ready};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use std::{
    future::{ready, Ready as StdReady},
    rc::Rc,
};

use crate::{models::user::UserClaims, AppState};

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

pub struct JwtMiddleware;

impl<S, B> Transform<S, ServiceRequest> for JwtMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = JwtMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ok(JwtMiddlewareService {
            service: Rc::new(service),
        })
    }
}

pub struct JwtMiddlewareService<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for JwtMiddlewareService<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();

        Box::pin(async move {
            let app_state = req.app_data::<web::Data<AppState>>();

            let state = app_state
                .ok_or_else(|| actix_web::error::ErrorInternalServerError("App state not found"))?;

            let token = extract_token_from_request(&req)
                .ok_or_else(|| actix_web::error::ErrorUnauthorized("JWT token required"))?;

            let claims = verify_jwt(&token, &state.config.jwt_secret)
                .map_err(|_| actix_web::error::ErrorUnauthorized("Invalid JWT token"))?;

            req.extensions_mut().insert(claims);

            service.call(req).await
        })
    }
}
