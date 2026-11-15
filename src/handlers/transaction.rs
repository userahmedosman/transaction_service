use crate::middleware::AuthenticatedBusiness;
use crate::models::{
    error::Result, CreateTransactionRequest, TransactionLogResponse, TransactionResponse,
};
use crate::services::TransactionService;
use axum::{
    extract::{Path, Query, State},
    Extension, Json,
};
use serde::Deserialize;
use sqlx::PgPool;
use uuid::Uuid;
use validator::Validate;

#[derive(Deserialize)]
pub struct ListQuery {
    #[serde(default = "default_limit")]
    limit: i64,
    #[serde(default)]
    offset: i64,
}

fn default_limit() -> i64 {
    50
}

pub async fn create_transaction(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Json(payload): Json<CreateTransactionRequest>,
) -> Result<Json<TransactionResponse>> {
    payload
        .validate()
        .map_err(|e| crate::models::AppError::ValidationError(e.to_string()))?;

    let service = TransactionService::new(pool);
    let transaction = service.create_transaction(auth.business_id, payload).await?;

    Ok(Json(transaction.into()))
}

pub async fn get_transaction(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<TransactionResponse>> {
    let service = TransactionService::new(pool);
    let transaction = service.get_transaction(auth.business_id, transaction_id).await?;

    Ok(Json(transaction.into()))
}

pub async fn list_transactions(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<TransactionResponse>>> {
    let service = TransactionService::new(pool);
    let transactions = service
        .list_transactions(auth.business_id, query.limit, query.offset)
        .await?;

    let response: Vec<TransactionResponse> = transactions.into_iter().map(|t| t.into()).collect();
    Ok(Json(response))
}

pub async fn get_transaction_logs(
    State(pool): State<PgPool>,
    Extension(auth): Extension<AuthenticatedBusiness>,
    Path(transaction_id): Path<Uuid>,
) -> Result<Json<Vec<TransactionLogResponse>>> {
    let service = TransactionService::new(pool);
    let logs = service
        .get_transaction_logs(auth.business_id, transaction_id)
        .await?;

    let response: Vec<TransactionLogResponse> = logs.into_iter().map(|l| l.into()).collect();
    Ok(Json(response))
}
