use actix_web::{web, HttpResponse, Result};
use serde_json::{json, Value};
use crate::config::Config;

/// Handler for GET /v1/models endpoint
/// Returns all available models from all providers with raw provider data
pub async fn models_endpoint(
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    match config.get_all_raw_models().await {
        Ok(all_models) => {
            let response = json!({
                "object": "list",
                "data": all_models
            });

            Ok(HttpResponse::Ok().json(response))
        }
        Err(e) => {
            eprintln!("Error fetching models: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": {
                    "message": format!("Failed to fetch models: {}", e),
                    "type": "internal_error"
                }
            })))
        }
    }
}

/// Handler for POST /v1/chat/completions endpoint
/// Forwards chat completion requests to the appropriate provider based on model name
pub async fn chat_completions(
    req: web::Json<Value>,
    config: web::Data<Config>,
) -> Result<HttpResponse> {
    // Extract model name from request
    let model = req.get("model")
        .and_then(|m| m.as_str())
        .ok_or_else(|| actix_web::error::ErrorBadRequest("Missing model field"))?;

    // Get model to provider mapping
    let model_mapping = config.get_model_mapping().await
        .map_err(|e| actix_web::error::ErrorInternalServerError(format!("Failed to get model mapping: {}", e)))?;

    // Find the provider for the requested model
    let provider = model_mapping.get(model)
        .ok_or_else(|| actix_web::error::ErrorNotFound(format!("Model '{}' not found", model)))?;

    // Create HTTP client and forward request
    let client = reqwest::Client::new();
    let url = format!("{}/chat/completions", provider.base_url.trim_end_matches('/'));

    let mut request_builder = client.post(&url).json(&req.into_inner());

    // Add authorization header if API key is provided
    if !provider.api_key.is_empty() {
        request_builder = request_builder.header("Authorization", format!("Bearer {}", provider.api_key));
    }

    // Send request and return response
    match request_builder.send().await {
        Ok(response) => {
            let status = response.status();
            let body = response.bytes().await.unwrap_or_default();

            // Convert reqwest status to actix status
            let actix_status = actix_web::http::StatusCode::from_u16(status.as_u16())
                .unwrap_or(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR);

            Ok(HttpResponse::build(actix_status)
                .content_type("application/json")
                .body(body))
        }
        Err(e) => {
            eprintln!("Error forwarding request: {}", e);
            Ok(HttpResponse::InternalServerError().json(json!({
                "error": {
                    "message": format!("Failed to forward request: {}", e),
                    "type": "internal_error"
                }
            })))
        }
    }
}