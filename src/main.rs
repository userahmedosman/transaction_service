mod handlers;
mod middleware;
mod models;
mod services;
mod utils;

use axum::{
    middleware as axum_middleware,
    routing::{delete, get, post},
    Router,
};
use sqlx::PgPool;
use std::net::SocketAddr;
use tower::ServiceBuilder;
use tower_http::{
    cors::CorsLayer,
    trace::{DefaultMakeSpan, DefaultOnResponse, TraceLayer},
};
use tracing::Level;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt()
        .with_target(false)
        .with_max_level(Level::DEBUG)
        .init();

    // Load environment variables
    dotenvy::dotenv().ok();

    let database_url = std::env::var("DATABASE_URL")
        .unwrap_or_else(|_| "postgresql://postgres:postgres@localhost:5432/transaction_service".to_string()); // Just for demo put here url with demo id and password    let host = std::env::var("SERVER_HOST").unwrap_or_else(|_| "0.0.0.0".to_string());
    let port = std::env::var("SERVER_PORT")
        .unwrap_or_else(|_| "1920".to_string())
        .parse::<u16>()?;

    // Create database connection pool
    tracing::info!("Connecting to database...");
    let pool = utils::create_pool(&database_url).await?;
    tracing::info!("Database connection established");

    // Build application
    let app = create_app(pool);

    // Start server
    let addr = SocketAddr::from(([0, 0, 0, 0], port));
    tracing::info!("Server listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

fn create_app(pool: PgPool) -> Router {
    // Public routes (no authentication required)
    let public_routes = Router::new()
        .route("/health", get(handlers::health_check))
        .route("/businesses", post(handlers::create_business))
        .route("/api-keys", post(handlers::create_api_key))
        .with_state(pool.clone());

    // Protected routes (authentication required)
    let routes = Router::new()
        // Account routes
        .route("/accounts", post(handlers::create_account))
        .route("/accounts", get(handlers::list_accounts))
        .route("/accounts/:id", get(handlers::get_account))
        .route("/accounts/:id/balance", get(handlers::get_account_balance))
        // Transaction routes
        .route("/transactions", post(handlers::create_transaction))
        .route("/transactions", get(handlers::list_transactions))
        .route("/transactions/:id", get(handlers::get_transaction))
        .route(
            "/transactions/:id/logs",
            get(handlers::get_transaction_logs),
        )
        // Webhook routes
        .route("/webhooks", post(handlers::create_webhook_endpoint))
        .route("/webhooks", get(handlers::list_webhook_endpoints))
        .route("/webhooks/:id", delete(handlers::delete_webhook_endpoint))
        .layer(
            ServiceBuilder::new()
                .layer(axum_middleware::from_fn_with_state(
                    pool.clone(),
                    middleware::auth_middleware,
                ))
                .layer(axum_middleware::from_fn_with_state(
                    pool.clone(),
                    middleware::rate_limit_middleware,
                )),
        )
        .with_state(pool.clone());

    // Combine routes
    // Combine routes
    Router::new()
        .merge(public_routes)
        .nest("/api/v1", routes)
        .layer(
            ServiceBuilder::new()
                .layer(
                    TraceLayer::new_for_http()
                        .make_span_with(DefaultMakeSpan::new().level(Level::INFO))
                        .on_response(DefaultOnResponse::new().level(Level::INFO)),
                )
                .layer(CorsLayer::permissive()),
        )
}