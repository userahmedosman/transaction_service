use crate::middleware::auth::AuthenticatedBusiness;
use crate::models::error::AppError;
use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};
use chrono::Utc;
use sqlx::PgPool;

pub async fn rate_limit_middleware(
    State(pool): State<PgPool>,
    request: Request,
    next: Next,
) -> Result<Response, AppError> {
    let auth_business = request
        .extensions()
        .get::<AuthenticatedBusiness>()
        .ok_or_else(|| AppError::Unauthorized("Not authenticated".to_string()))?
        .clone();

    // Get API key rate limit
    let api_key = sqlx::query_as::<_, (i32,)>(
        r#"SELECT rate_limit FROM api_keys WHERE id = $1"#,
    )
    .bind(auth_business.api_key_id)
    .fetch_one(&pool)
    .await?;

    let rate_limit = api_key.0;

    // Check current token bucket
    let mut tx = pool.begin().await?;

    let bucket = sqlx::query_as::<_, (i32, chrono::DateTime<chrono::Utc>)>(
        r#"
        SELECT tokens, last_refill_at 
        FROM rate_limit_buckets 
        WHERE api_key_id = $1 
        FOR UPDATE
        "#,
    )
    .bind(auth_business.api_key_id)
    .fetch_optional(&mut *tx)
    .await?;

    let now = Utc::now();
    let (tokens, last_refill) = match bucket {
        Some((tokens, last_refill_at)) => {
            // Calculate token refill
            let elapsed = (now - last_refill_at).num_seconds();
            let refill_amount = (elapsed / 3600) * rate_limit as i64; // Refill hourly
            let new_tokens = (tokens as i64 + refill_amount).min(rate_limit as i64);
            (new_tokens as i32, last_refill_at)
        }
        None => {
            // Initialize bucket
            sqlx::query(
                r#"
                INSERT INTO rate_limit_buckets (api_key_id, tokens, last_refill_at)
                VALUES ($1, $2, NOW())
                "#,
            )
            .bind(auth_business.api_key_id)
            .bind(rate_limit)
            .execute(&mut *tx)
            .await?;
            (rate_limit, now)
        }
    };

    if tokens <= 0 {
        tx.rollback().await?;
        return Err(AppError::RateLimitExceeded(
            "Rate limit exceeded".to_string(),
        ));
    }

    // Consume a token
    let new_tokens = tokens - 1;
    sqlx::query(
        r#"
        UPDATE rate_limit_buckets 
        SET tokens = $1, updated_at = NOW()
        WHERE api_key_id = $2
        "#,
    )
    .bind(new_tokens)
    .bind(auth_business.api_key_id)
    .execute(&mut *tx)
    .await?;

    tx.commit().await?;

    Ok(next.run(request).await)
}
