use crate::models::{error::AppError, ApiKey};
use crate::utils::ApiKeyGenerator;
use axum::{
    extract::{Request, State},
    http::StatusCode,
    middleware::Next,
    response::Response,
};
use sqlx::PgPool;
use uuid::Uuid;

#[derive(Clone)]
pub struct AuthenticatedBusiness {
    pub business_id: Uuid,
    pub api_key_id: Uuid,
}

pub async fn auth_middleware(
    State(pool): State<PgPool>,
    mut request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_header = request
        .headers()
        .get("Authorization")
        .and_then(|h| h.to_str().ok())
        .ok_or_else(|| AppError::Unauthorized("Missing Authorization header".to_string()))?;

    let api_key = auth_header
        .strip_prefix("Bearer ")
        .ok_or_else(|| AppError::Unauthorized("Invalid Authorization format".to_string()))?;

    // Extract prefix to speed up lookup
    let prefix = api_key.chars().take(12).collect::<String>();

    // Look up API key by prefix
    let key_record = sqlx::query_as::<_, ApiKey>(
        r#"
        SELECT * FROM api_keys 
        WHERE key_prefix = $1 AND is_active = true
        "#,
    )
    .bind(&prefix)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| AppError::Unauthorized("Invalid API key".to_string()))?;

    // Verify the full key hash
    if !ApiKeyGenerator::verify_key(api_key, &key_record.key_hash) {
        return Err(AppError::Unauthorized("Invalid API key".to_string()));
    }

    // Check if key is expired
    if let Some(expires_at) = key_record.expires_at {
        if expires_at < chrono::Utc::now() {
            return Err(AppError::Unauthorized("API key expired".to_string()));
        }
    }

    // Update last_used_at
    let _ = sqlx::query(
        r#"
        UPDATE api_keys SET last_used_at = NOW() WHERE id = $1
        "#,
    )
    .bind(key_record.id)
    .execute(&pool)
    .await;

    // Store authenticated business info in request extensions
    request.extensions_mut().insert(AuthenticatedBusiness {
        business_id: key_record.business_id,
        api_key_id: key_record.id,
    });

    Ok(next.run(request).await)
}
