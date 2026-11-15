use axum::{extract::State, Json};
use sqlx::PgPool;

pub async fn health_check(State(pool): State<PgPool>) -> Json<serde_json::Value> {
    let db_status = match sqlx::query("SELECT 1").execute(&pool).await {
        Ok(_) => "healthy",
        Err(_) => "unhealthy",
    };

    Json(serde_json::json!({
        "status": "ok",
        "database": db_status,
        "service": "transaction-service",
        "version": env!("CARGO_PKG_VERSION"),
    }))
}
