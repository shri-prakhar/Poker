use crate::auth::jwt::validate_token;
use crate::errors::ServiceError;
use crate::state::AppState;
use actix_web::dev::{Service, ServiceRequest, ServiceResponse, Transform};
use actix_web::http::header::{AUTHORIZATION, REFRESH};
use actix_web::web::Data;
use actix_web::{Error, HttpMessage};
use futures::future::LocalBoxFuture;
use std::future::{Ready, ready};
use std::rc::Rc;

pub struct AuthMiddleware;

impl AuthMiddleware {
    pub fn new() -> Self {
        Self
    }
}

impl<S, B> Transform<S, ServiceRequest> for AuthMiddleware
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + Send
        + Sync
        + 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Transform = AuthMiddlewareMiddleware<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        Ready(Ok(AuthMiddlewareMiddleware {
            service: Rc::new(service),
        }))
    }
}

pub struct AuthMiddlewareMiddleware<S> {
    service: Rc<S>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>
        + Send
        + Sync
        + 'static,
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

    fn call(&self, mut req: ServiceRequest) -> Self::Future {
        let srv = self.service.clone();
        Box::pin(async move {
            let app_state = match req.app_data::<Data<AppState>>() {
                Some(d) => d.clone(),
                None => {
                    return Err(actix_web::error::ErrorInternalServerError(
                        "app state error",
                    ));
                }
            };
            let token_option = req
                .headers()
                .get(AUTHORIZATION)
                .and_then(|h| h.to_str().ok())
                .and_then(|s| {
                    let s = s.trim(); //removes the initial and trailing whitespaces  
                    if s.to_ascii_lowercase().starts_with("bearer ") {
                        Some(s[7..].trim().to_string())
                    } else {
                        None
                    }
                });

            let token = match token_option {
                Some(tk) => tk,
                None => {
                    return Err(actix_web::error::ErrorUnauthorized(
                        "Missing Authorization header",
                    ));
                }
            };

            let token_data = match validate_token(&token, &app_state.setting.jwt_secret) {
                Ok(tk) => tk,
                Err(_) => return Err(actix_web::error::ErrorUnauthorized("Invalid Token")),
            };

            req.extensions_mut().insert(token_data.claims);
            let response = srv.call(req).await?;
            Ok(response)
        })
    }
}
