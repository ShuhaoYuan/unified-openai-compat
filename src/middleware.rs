use std::future::{ready, Ready};

use actix_web::{
    dev::{forward_ready, Service, ServiceRequest, ServiceResponse, Transform},
    Error, web,
    http::header::AUTHORIZATION,
};
use futures_util::future::LocalBoxFuture;
use crate::config::Config;

pub struct ApiKeyAuth;

// Middleware factory is `Transform` trait
// `S` - type of the next service
// `B` - type of response's body
impl<S, B> Transform<S, ServiceRequest> for ApiKeyAuth
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = ApiKeyAuthMiddleware<S>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(ApiKeyAuthMiddleware { service }))
    }
}

pub struct ApiKeyAuthMiddleware<S> {
    service: S,
}

impl<S, B> Service<ServiceRequest> for ApiKeyAuthMiddleware<S>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error>,
    S::Future: 'static,
    B: 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    forward_ready!(service);

    fn call(&self, req: ServiceRequest) -> Self::Future {
        // Get the configuration from app data
        let config = req.app_data::<web::Data<Config>>().map(|data| data.as_ref().clone());
        let path = req.path().to_string();
        
        println!("Middleware: Processing request to {}", path);
        
        // Skip authentication for /v1/models endpoint (optional)
        if path == "/v1/models" {
            println!("Middleware: Skipping authentication for /v1/models");
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        // If no config is provided, skip authentication (for development)
        if config.is_none() {
            println!("Middleware: No config found, skipping authentication");
            let fut = self.service.call(req);
            return Box::pin(async move {
                let res = fut.await?;
                Ok(res)
            });
        }

        let config = config.unwrap();
        println!("Middleware: Config found, server_api_key: {:?}", config.server_api_key);
        
        // Extract API key from Authorization header
        let auth_header = req.headers().get(AUTHORIZATION).cloned();
        
        let api_key_valid = match auth_header {
            Some(header_value) => {
                if let Ok(auth_str) = header_value.to_str() {
                    println!("Middleware: Found auth header: {}", auth_str);
                    // Check for "Bearer " prefix
                    if auth_str.starts_with("Bearer ") {
                        let provided_key = &auth_str[7..]; // Remove "Bearer " prefix
                        println!("Middleware: Extracted API key: {}", provided_key);
                        let is_valid = config.validate_api_key(provided_key);
                        println!("Middleware: API key validation result: {}", is_valid);
                        is_valid
                    } else {
                        println!("Middleware: No Bearer prefix found");
                        false
                    }
                } else {
                    println!("Middleware: Invalid auth header format");
                    false
                }
            }
            None => {
                println!("Middleware: No auth header found");
                false
            }
        };

        if !api_key_valid {
            println!("Middleware: Authentication failed, returning 401");
            // Return 401 Unauthorized if API key is invalid
            return Box::pin(async move {
                Err(actix_web::error::ErrorUnauthorized(serde_json::json!({
                    "error": {
                        "message": "Invalid API key",
                        "type": "authentication_error"
                    }
                })))
            });
        }

        println!("Middleware: Authentication successful, proceeding to service");
        let fut = self.service.call(req);
        Box::pin(async move {
            let res = fut.await?;
            Ok(res)
        })
    }
}
