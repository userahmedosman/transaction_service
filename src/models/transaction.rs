use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Transaction {
    pub id: Uuid,
    pub idempotency_key: String,
    pub business_id: Uuid,
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub source_account_id: Option<Uuid>,
    pub destination_account_id: Option<Uuid>,
    pub status: String,
    pub description: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateTransactionRequest {
    #[validate(length(min = 1, max = 255))]
    pub idempotency_key: String,
    #[validate(length(min = 1, max = 20))]
    pub transaction_type: String,
    #[validate(range(min = 1))]
    pub amount: i64,
    #[validate(length(equal = 3))]
    pub currency: String,
    pub source_account_id: Option<Uuid>,
    pub destination_account_id: Option<Uuid>,
    pub description: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
pub struct TransactionResponse {
    pub id: Uuid,
    pub idempotency_key: String,
    pub transaction_type: String,
    pub amount: i64,
    pub currency: String,
    pub source_account_id: Option<Uuid>,
    pub destination_account_id: Option<Uuid>,
    pub status: String,
    pub description: Option<String>,
    pub metadata: Option<JsonValue>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error_message: Option<String>,
}

impl From<Transaction> for TransactionResponse {
    fn from(transaction: Transaction) -> Self {
        TransactionResponse {
            id: transaction.id,
            idempotency_key: transaction.idempotency_key,
            transaction_type: transaction.transaction_type,
            amount: transaction.amount,
            currency: transaction.currency,
            source_account_id: transaction.source_account_id,
            destination_account_id: transaction.destination_account_id,
            status: transaction.status,
            description: transaction.description,
            metadata: transaction.metadata,
            created_at: transaction.created_at,
            completed_at: transaction.completed_at,
            error_message: transaction.error_message,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct TransactionLog {
    pub id: i64,
    pub transaction_id: Uuid,
    pub previous_status: Option<String>,
    pub new_status: String,
    pub changed_by: Option<String>,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
    pub metadata: Option<JsonValue>,
}

#[derive(Debug, Serialize)]
pub struct TransactionLogResponse {
    pub id: i64,
    pub previous_status: Option<String>,
    pub new_status: String,
    pub changed_at: DateTime<Utc>,
    pub reason: Option<String>,
}

impl From<TransactionLog> for TransactionLogResponse {
    fn from(log: TransactionLog) -> Self {
        TransactionLogResponse {
            id: log.id,
            previous_status: log.previous_status,
            new_status: log.new_status,
            changed_at: log.changed_at,
            reason: log.reason,
        }
    }
}
