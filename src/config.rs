use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Represents a model provider configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    pub base_url: String,
    pub api_key: String,
    /// Optional static models configuration for this provider
    /// Can be either a simple string array or detailed ModelInfo objects
    /// If provided, these models will be used instead of fetching from the provider's /models endpoint
    pub models: Option<Vec<String>>,
}


/// Main configuration structure containing all providers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Optional API key for the unified server
    /// If not set, the server will not require authentication
    pub server_api_key: Option<String>,
    /// List of model providers
    pub providers: Vec<Provider>,
}

impl Config {
    /// Load configuration from config.toml file
    pub fn load() -> Result<Self, Box<dyn std::error::Error>> {
        let config_content = std::fs::read_to_string("config.toml")?;
        let config: Config = toml::from_str(&config_content)?;
        Ok(config)
    }

    /// Get model to provider mapping with priority handling
    pub async fn get_model_mapping(&self) -> Result<HashMap<String, Provider>, Box<dyn std::error::Error>> {
        let mut mapping = HashMap::new();
        let mut seen_models = std::collections::HashSet::new();

        // Process providers in order (top to bottom priority)
        for provider in &self.providers {
            let models = self.fetch_models_from_provider(provider).await?;
            for model in models {
                // Only add model if we haven't seen it before (priority logic)
                if !seen_models.contains(&model) {
                    mapping.insert(model.clone(), provider.clone());
                    seen_models.insert(model);
                }
            }
        }
        Ok(mapping)
    }

    /// Fetch model names from a specific provider
    /// If static models are configured, use them; otherwise fetch from provider's /models endpoint
    pub async fn fetch_models_from_provider(&self, provider: &Provider) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // If static models are configured, use them
        if let Some(static_models) = &provider.models {
            println!("Using static models configuration for provider: {}", provider.base_url);
            return Ok(static_models.clone());
        }

        // Otherwise, fetch from provider's /models endpoint
        let client = reqwest::Client::new();
        let url = format!("{}/models", provider.base_url.trim_end_matches('/'));

        let mut request_builder = client.get(&url);

        // Add authorization header if API key is provided
        if !provider.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", provider.api_key));
        }

        match request_builder.send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!("Warning: Failed to fetch models from {}: {}", provider.base_url, response.status());
                    return Ok(Vec::new()); // Return empty list instead of error
                }

                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        let mut models = Vec::new();

                        // Extract model IDs from the response
                        if let Some(data) = json_response.get("data").and_then(|d| d.as_array()) {
                            for model in data {
                                if let Some(model_id) = model.get("id").and_then(|id| id.as_str()) {
                                    models.push(model_id.to_string());
                                }
                            }
                        }

                        Ok(models)
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse models response from {}: {}", provider.base_url, e);
                        Ok(Vec::new()) // Return empty list instead of error
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to connect to {}: {}", provider.base_url, e);
                Ok(Vec::new()) // Return empty list instead of error
            }
        }
    }


    /// Get all models with raw provider data
    pub async fn get_all_raw_models(&self) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        let mut all_models = Vec::new();
        let mut seen_models = std::collections::HashSet::new();

        // Process providers in order (top to bottom priority)
        for provider in &self.providers {
            let models = self.fetch_raw_models_from_provider(provider).await?;
            for model in models {
                if let Some(model_id) = model.get("id").and_then(|id| id.as_str()) {
                    // Only add model if we haven't seen it before (priority logic)
                    if !seen_models.contains(model_id) {
                        all_models.push(model.clone());
                        seen_models.insert(model_id.to_string());
                    }
                }
            }
        }

        Ok(all_models)
    }

    /// Fetch raw model objects from a specific provider
    /// If static models are configured, use them; otherwise fetch from provider's /models endpoint
    pub async fn fetch_raw_models_from_provider(&self, provider: &Provider) -> Result<Vec<serde_json::Value>, Box<dyn std::error::Error>> {
        // If static models are configured, use them
        if let Some(static_models) = &provider.models {
            println!("Using static models configuration for provider: {}", provider.base_url);
            let mut models = Vec::new();
            for model_id in static_models {
                let model_json = serde_json::json!({
                    "id": model_id,
                    "object": "model",
                    "created": null,
                    "owned_by": null
                });
                models.push(model_json);
            }
            return Ok(models);
        }

        // Otherwise, fetch from provider's /models endpoint
        let client = reqwest::Client::new();
        let url = format!("{}/models", provider.base_url.trim_end_matches('/'));

        let mut request_builder = client.get(&url);

        // Add authorization header if API key is provided
        if !provider.api_key.is_empty() {
            request_builder = request_builder.header("Authorization", format!("Bearer {}", provider.api_key));
        }

        match request_builder.send().await {
            Ok(response) => {
                if !response.status().is_success() {
                    eprintln!("Warning: Failed to fetch models from {}: {}", provider.base_url, response.status());
                    return Ok(Vec::new()); // Return empty list instead of error
                }

                match response.json::<serde_json::Value>().await {
                    Ok(json_response) => {
                        let mut models = Vec::new();

                        // Extract complete model objects from the response
                        if let Some(data) = json_response.get("data").and_then(|d| d.as_array()) {
                            for model in data {
                                models.push(model.clone());
                            }
                        }

                        Ok(models)
                    }
                    Err(e) => {
                        eprintln!("Warning: Failed to parse models response from {}: {}", provider.base_url, e);
                        Ok(Vec::new()) // Return empty list instead of error
                    }
                }
            }
            Err(e) => {
                eprintln!("Warning: Failed to connect to {}: {}", provider.base_url, e);
                Ok(Vec::new()) // Return empty list instead of error
            }
        }
    }

    /// Validate the provided API key against the configured server API key
    /// Returns true if authentication is disabled or if the key matches
    pub fn validate_api_key(&self, provided_key: &str) -> bool {
        match &self.server_api_key {
            Some(configured_key) => {
                // If server API key is configured, validate against it
                provided_key == configured_key
            }
            None => {
                // If no server API key is configured, allow all requests (development mode)
                true
            }
        }
    }
}
