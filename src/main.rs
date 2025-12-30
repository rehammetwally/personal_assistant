mod api;
mod auth;
mod db;
mod groq;
mod models;

use std::net::SocketAddr;
use tower_http::cors::CorsLayer;
use tower_http::services::ServeDir;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use crate::api::{create_router, AppState};
use crate::db::init_db;
use crate::groq::GroqClient;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::registry()
        .with(tracing_subscriber::EnvFilter::new(
            std::env::var("RUST_LOG").unwrap_or_else(|_| "info".to_string()),
        ))
        .with(tracing_subscriber::fmt::layer())
        .init();

    dotenv::dotenv().ok();

    println!("🚀 Starting SaaS Personal Assistant Backend...");

    // Initialize Database
    let pool = init_db().await.map_err(|e| {
        eprintln!("❌ Database error: {}", e);
        e
    })?;
    println!("✅ Database connected & initialized.");

    // Initialize Groq AI
    let groq_client = match GroqClient::new() {
        Ok(client) => {
            println!("✅ Groq AI connected successfully!");
            Some(client)
        }
        Err(e) => {
            println!("⚠️  Warning: Groq AI disabled - {}", e);
            None
        }
    };

    let state = AppState {
        db: pool,
        groq: groq_client,
    };

    // Build router
    let app = create_router(state)
        .fallback_service(ServeDir::new("frontend"))
        .layer(CorsLayer::permissive());

    // Start server
    let addr = SocketAddr::from(([127, 0, 0, 1], 3000));
    println!("📡 Server listening on http://{}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}
