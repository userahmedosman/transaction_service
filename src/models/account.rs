use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Account {
    pub id: Uuid,
    pub business_id: Uuid,
    pub account_number: String,
    pub account_type: String,
    pub currency: String,
    pub balance: i64,
    pub available_balance: i64,
    pub version: i32,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateAccountRequest {
    #[validate(length(min = 1, max = 20))]
    pub account_type: String,
    #[validate(length(equal = 3))]
    pub currency: String,
}

#[derive(Debug, Serialize)]
pub struct AccountResponse {
    pub id: Uuid,
    pub account_number: String,
    pub account_type: String,
    pub currency: String,
    pub balance: i64,
    pub available_balance: i64,
    pub created_at: DateTime<Utc>,
    pub status: String,
}

impl From<Account> for AccountResponse {
    fn from(account: Account) -> Self {
        AccountResponse {
            id: account.id,
            account_number: account.account_number,
            account_type: account.account_type,
            currency: account.currency,
            balance: account.balance,
            available_balance: account.available_balance,
            created_at: account.created_at,
            status: account.status,
        }
    }
}
