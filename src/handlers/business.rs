use crate::models::{
    error::Result, Business, BusinessResponse, CreateBusinessRequest, CreateApiKeyRequest,
    CreateApiKeyResponse, ApiKeyResponse, ApiKey,
};
use crate::utils::ApiKeyGenerator;
use axum::{extract::State, Json};
use sqlx::PgPool;
use validator::Validate;

pub async fn create_business(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateBusinessRequest>,
) -> Result<Json<BusinessResponse>> {
    payload
        .validate()
        .map_err(|e| crate::models::AppError::ValidationError(e.to_string()))?;

    let business = sqlx::query_as::<_, Business>(
        r#"
        INSERT INTO businesses (name, email, status)
        VALUES ($1, $2, 'active')
        RETURNING *
        "#,
    )
    .bind(&payload.name)
    .bind(&payload.email)
    .fetch_one(&pool)
    .await?;

    Ok(Json(business.into()))
}

// pub async fn create_api_key(
//     State(pool): State<PgPool>,
//     Json(payload): Json<CreateApiKeyRequest>,
// ) -> Result<Json<CreateApiKeyResponse>> {
//     payload
//         .validate()
//         .map_err(|e| crate::models::AppError::ValidationError(e.to_string()))?;

//     // For demo purposes, assume business_id is passed in headers or from a previous registration
//     // In production, this would come from an authenticated session
//     let business_id = uuid::Uuid::new_v4(); // Replace with actual business_id

//     let (key, prefix, hash) = ApiKeyGenerator::generate();

//     let api_key = sqlx::query_as::<_, ApiKey>(
//         r#"
//         INSERT INTO api_keys 
//         (business_id, key_hash, key_prefix, name, rate_limit, expires_at, is_active)
//         VALUES ($1, $2, $3, $4, $5, $6, true)
//         RETURNING *
//         "#,
//     )
//     .bind(business_id)
//     .bind(&hash)
//     .bind(&prefix)
//     .bind(&payload.name)
//     .bind(payload.rate_limit.unwrap_or(1000))
//     .bind(payload.expires_at)
//     .fetch_one(&pool)
//     .await?;

//     Ok(Json(CreateApiKeyResponse {
//         id: api_key.id,
//         name: api_key.name,
//         key,
//         key_prefix: api_key.key_prefix,
//         created_at: api_key.created_at,
//         expires_at: api_key.expires_at,
//         rate_limit: api_key.rate_limit,
//     }))
// }

pub async fn create_api_key(
    State(pool): State<PgPool>,
    Json(payload): Json<CreateApiKeyRequest>,
) -> Result<Json<CreateApiKeyResponse>> {
    payload
        .validate()
        .map_err(|e| crate::models::AppError::ValidationError(e.to_string()))?;

    // Use business_id from request instead of generating random UUID
    let business_id = payload.business_id;  // ← CHANGE THIS LINE
    
    // Verify business exists (optional but recommended)
    let business_exists = sqlx::query_as::<_, (bool,)>(
        r#"SELECT EXISTS(SELECT 1 FROM businesses WHERE id = $1)"#,
    )
    .bind(business_id)
    .fetch_one(&pool)
    .await?;

    if !business_exists.0 {
        return Err(crate::models::AppError::NotFound(
            "Business not found".to_string()
        ));
    }

    let (key, prefix, hash) = ApiKeyGenerator::generate();

    let api_key = sqlx::query_as::<_, ApiKey>(
        r#"
        INSERT INTO api_keys 
        (business_id, key_hash, key_prefix, name, rate_limit, expires_at, is_active)
        VALUES ($1, $2, $3, $4, $5, $6, true)
        RETURNING *
        "#,
    )
    .bind(business_id)
    .bind(&hash)
    .bind(&prefix)
    .bind(&payload.name)
    .bind(payload.rate_limit.unwrap_or(1000))
    .bind(payload.expires_at)
    .fetch_one(&pool)
    .await?;

    Ok(Json(CreateApiKeyResponse {
        id: api_key.id,
        name: api_key.name,
        key,
        key_prefix: api_key.key_prefix,
        created_at: api_key.created_at,
        expires_at: api_key.expires_at,
        rate_limit: api_key.rate_limit,
    }))
}