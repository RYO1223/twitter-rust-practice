use actix_web::{
    Error, HttpMessage,
    dev::{Service, ServiceRequest, ServiceResponse, Transform, forward_ready},
    error::ErrorUnauthorized,
    http::header,
};
use futures_util::future::{LocalBoxFuture, Ready, ready};
use jwt_simple::prelude::*;
use serde::{Deserialize, Serialize};
use std::{env, rc::Rc};

// Claims structure for JWT payload
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct AuthClaims {
    pub user_id: i32,
    pub username: String,
}

// Authentication utility
pub struct Authentication;

impl Authentication {
    // Function to create a new JWT token
    pub fn create_token(user_id: i32, username: &str) -> Result<String, String> {
        let key = get_jwt_key();

        let claims = Claims::with_custom_claims(
            AuthClaims {
                user_id,
                username: username.to_owned(),
            },
            Duration::from_days(7),
        );

        key.authenticate(claims)
            .map_err(|e| format!("Error creating token: {}", e))
    }

    // Function to verify and extract claims from token
    pub fn verify_token(token: &str) -> Result<JWTClaims<AuthClaims>, String> {
        let key = get_jwt_key();

        key.verify_token::<AuthClaims>(token, None)
            .map_err(|e| format!("Error verifying token: {}", e))
    }
}

// Helper function to get JWT key
fn get_jwt_key() -> HS256Key {
    let secret = env::var("JWT_SECRET").expect("JWT_SECRET must be set in .env file");
    HS256Key::from_bytes(secret.as_bytes())
}

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
        match Authentication::verify_token(token) {
            Ok(claims) => {
                // Insert auth claims into request extensions
                req.extensions_mut().insert(claims.custom);

                let fut = service.call(req);
                Box::pin(async move {
                    let res = fut.await?;
                    Ok(res)
                })
            }
            Err(_) => Box::pin(async move { Err(ErrorUnauthorized("Invalid or expired token")) }),
        }
    }
}

// Extension trait to extract auth claims from requests
pub trait AuthenticatedRequest {
    fn get_auth_claims(&self) -> Option<AuthClaims>;
}

impl AuthenticatedRequest for ServiceRequest {
    fn get_auth_claims(&self) -> Option<AuthClaims> {
        self.extensions().get::<AuthClaims>().cloned()
    }
}
