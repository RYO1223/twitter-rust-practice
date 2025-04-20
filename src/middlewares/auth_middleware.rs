use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::ErrorUnauthorized,
    http::header,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use std::rc::Rc;

use crate::{models::user::AuthedUserId, util::auth::Authentication};

// Auth middleware factory
pub struct AuthMiddleware {
    pub ignore_routes: Vec<String>,
}

impl AuthMiddleware {
    pub fn new() -> Self {
        Self {
            ignore_routes: Vec::new(),
        }
    }

    pub fn ignore(mut self, route: &str) -> Self {
        self.ignore_routes.push(route.to_string());
        self
    }
}

impl Default for AuthMiddleware {
    fn default() -> Self {
        Self::new()
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
    type Transform = AuthMiddlewareService<S>;
    type InitError = ();
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(AuthMiddlewareService {
            service: Rc::new(service),
            ignore_routes: self.ignore_routes.clone(),
        }))
    }
}

pub struct AuthMiddlewareService<S> {
    service: Rc<S>,
    ignore_routes: Vec<String>,
}

impl<S, B> Service<ServiceRequest> for AuthMiddlewareService<S>
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
        let service = Rc::clone(&self.service);

        // Get the path to check against ignored routes
        let path = req.path().to_string();

        // Skip auth for OPTIONS requests (CORS preflight)
        // or if the route is in the ignore list
        if req.method() == "OPTIONS" || self.ignore_routes.iter().any(|r| path.starts_with(r)) {
            return Box::pin(async move {
                let res = service.call(req).await?;
                Ok(res)
            });
        }

        // Check for authorization header
        let auth_header = match req.headers().get(header::AUTHORIZATION) {
            Some(header) => header.to_str().unwrap_or_default(),
            None => {
                return Box::pin(
                    async move { Err(ErrorUnauthorized("Authorization header missing")) },
                );
            }
        };

        // Extract token from header (format: "Bearer <token>")
        let token = match auth_header.strip_prefix("Bearer ") {
            Some(token) => token,
            None => {
                return Box::pin(
                    async move { Err(ErrorUnauthorized("Invalid authorization format")) },
                );
            }
        };

        // Verify token
        let claims = match Authentication::verify_token(token) {
            Ok(claims) => claims,

            Err(_) => {
                return Box::pin(async move { Err(ErrorUnauthorized("Invalid or expired token")) });
            }
        };

        let user_id = match claims.subject {
            Some(user_id_string) => user_id_string
                .parse::<i32>()
                .expect("Invalid user ID in token"),

            None => {
                return Box::pin(async move { Err(ErrorUnauthorized("Invalid token claims")) });
            }
        };

        // Token is valid, proceed with the request
        req.extensions_mut().insert(AuthedUserId(user_id));

        let fut = service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
