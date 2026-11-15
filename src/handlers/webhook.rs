use crate::middleware::AuthenticatedBusiness;
use crate::models::{
    error::Result, CreateWebhookEndpointRequest, WebhookEndpoint, WebhookEndpointResponse,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub async fn create_webhook_endpoint(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Json(payload): Json<CreateWebhookEndpointRequest>,
) -> Result<Json<WebhookEndpointResponse>> {
    payload
        .validate()
        .map_err(|e| crate::models::AppError::ValidationError(e.to_string()))?;

    // Generate webhook secret
    let secret = format!("whsec_{}", Uuid::new_v4().to_string().replace("-", ""));

    let endpoint = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        INSERT INTO webhook_endpoints 
        (business_id, url, secret, events, is_active, max_retries, retry_delay_seconds)
        VALUES ($1, $2, $3, $4, true, 3, 60)
        RETURNING *
        "#,
    )
    .bind(auth.business_id)
    .bind(&payload.url)
    .bind(&secret)
    .bind(&payload.events)
    .fetch_one(&pool)
    .await?;

    Ok(Json(endpoint.into()))
}

pub async fn list_webhook_endpoints(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
) -> Result<Json<Vec<WebhookEndpointResponse>>> {
    let endpoints = sqlx::query_as::<_, WebhookEndpoint>(
        r#"
        SELECT * FROM webhook_endpoints 
        WHERE business_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth.business_id)
    .fetch_all(&pool)
    .await?;

    let response: Vec<WebhookEndpointResponse> =
        endpoints.into_iter().map(|e| e.into()).collect();
    Ok(Json(response))
}

pub async fn delete_webhook_endpoint(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Path(endpoint_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let result = sqlx::query(
        r#"
        DELETE FROM webhook_endpoints 
        WHERE id = $1 AND business_id = $2
        "#,
    )
    .bind(endpoint_id)
    .bind(auth.business_id)
    .execute(&pool)
    .await?;

    if result.rows_affected() == 0 {
        return Err(crate::models::AppError::NotFound(
            "Webhook endpoint not found".to_string(),
        ));
    }

    Ok(Json(serde_json::json!({
        "message": "Webhook endpoint deleted successfully"
    })))
}
