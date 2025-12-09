use crate::auth::jwt::validate_token;
use crate::state::AppState;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::AUTHORIZATION;
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(
        &self,
        ctx: &mut core::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.service.poll_ready(ctx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let srv = Rc::clone(&self.service);

        Box::pin(async move {
            let app_state = match req.app_data::<Data<AppState>>() {
                Some(data) => data.clone(),
                None => {
                    return Err(actix_web::error::ErrorInternalServerError(
                        "app state missing",
                    ));
                }
            };

            let token = match req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| {
                    let s = s.trim();
                    if s.to_ascii_lowercase().starts_with("bearer ") {
                        Some(s[7..].trim().to_string())
                    } else {
                        None
                    }
                }) {
                Some(t) => t,
                None => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Missing Authorization header",
                    ));
                }
            };

            let token_data = match validate_token(&token, &app_state.setting.jwt_secret) {
                Ok(data) => data,
                Err(_) => {
                    return Err(actix_web::error::ErrorUnauthorized("Invalid Token"));
                }
            };

            req.extensions_mut().insert(token_data.claims);

            let res = srv.call(req).await?;
            Ok(res)
        })
    }
}
