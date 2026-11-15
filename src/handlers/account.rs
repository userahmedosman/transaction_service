use crate::middleware::AuthenticatedBusiness;
use crate::models::{
    error::Result, Account, AccountResponse, CreateAccountRequest,
};
use axum::{
    extract::{Path, State},
    Extension, Json,
};
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

pub async fn create_account(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Json(payload): Json<CreateAccountRequest>,
) -> Result<Json<AccountResponse>> {
    payload
        .validate()
        .map_err(|e| crate::models::AppError::ValidationError(e.to_string()))?;

    // Generate unique account number
    let account_number = format!("ACC{}", Uuid::new_v4().to_string().replace("-", ""));

    let account = sqlx::query_as::<_, Account>(
        r#"
        INSERT INTO accounts 
        (business_id, account_number, account_type, currency, balance, 
         available_balance, version, status)
        VALUES ($1, $2, $3, $4, 0, 0, 1, 'active')
        RETURNING *
        "#,
    )
    .bind(auth.business_id)
    .bind(&account_number)
    .bind(&payload.account_type)
    .bind(&payload.currency)
    .fetch_one(&pool)
    .await?;

    Ok(Json(account.into()))
}

pub async fn get_account(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<AccountResponse>> {
    let account = sqlx::query_as::<_, Account>(
        r#"
        SELECT * FROM accounts 
        WHERE id = $1 AND business_id = $2
        "#,
    )
    .bind(account_id)
    .bind(auth.business_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| crate::models::AppError::NotFound("Account not found".to_string()))?;

    Ok(Json(account.into()))
}

pub async fn list_accounts(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
) -> Result<Json<Vec<AccountResponse>>> {
    let accounts = sqlx::query_as::<_, Account>(
        r#"
        SELECT * FROM accounts 
        WHERE business_id = $1
        ORDER BY created_at DESC
        "#,
    )
    .bind(auth.business_id)
    .fetch_all(&pool)
    .await?;

    let response: Vec<AccountResponse> = accounts.into_iter().map(|a| a.into()).collect();
    Ok(Json(response))
}

pub async fn get_account_balance(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Path(account_id): Path<Uuid>,
) -> Result<Json<serde_json::Value>> {
    let account = sqlx::query_as::<_, Account>(
        r#"
        SELECT * FROM accounts 
        WHERE id = $1 AND business_id = $2
        "#,
    )
    .bind(account_id)
    .bind(auth.business_id)
    .fetch_optional(&pool)
    .await?
    .ok_or_else(|| crate::models::AppError::NotFound("Account not found".to_string()))?;

    Ok(Json(serde_json::json!({
        "account_id": account.id,
        "balance": account.balance,
        "available_balance": account.available_balance,
        "currency": account.currency,
    })))
}
