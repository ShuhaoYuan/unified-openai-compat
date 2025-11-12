// Module declarations
mod config;
mod handlers;
mod middleware;

use actix_web::{web, App, HttpServer, middleware::Logger};
use config::Config;
use middleware::ApiKeyAuth;

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Initialize logger
    env_logger::init();

    // Load configuration from file
    let config = Config::load().expect("Failed to load configuration");

    println!("Starting unified OpenAI compatible server...");
    
    // Print authentication status
    match &config.server_api_key {
        Some(_) => println!("🔒 API key authentication: ENABLED"),
        None => println!("🔓 API key authentication: DISABLED (development mode)"),
    }
    
    println!("Configured providers:");
    for (i, provider) in config.providers.iter().enumerate() {
        println!("  {}. {} (priority: {})", i + 1, provider.base_url, i + 1);
    }

    // Create and run HTTP server
    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(config.clone()))
            .wrap(Logger::default())
            .wrap(ApiKeyAuth) // Add API key authentication middleware
            .route("/v1/models", web::get().to(handlers::models_endpoint))
            .route("/v1/chat/completions", web::post().to(handlers::chat_completions))
    })
    .bind("127.0.0.1:8080")?
    .run()
    .await
}