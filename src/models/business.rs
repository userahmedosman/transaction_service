use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Business {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub status: String,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateBusinessRequest {
    #[validate(length(min = 1, max = 255))]
    pub name: String,
    #[validate(email)]
    pub email: String,
}

#[derive(Debug, Serialize)]
pub struct BusinessResponse {
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub status: String,
}

impl From<Business> for BusinessResponse {
    fn from(business: Business) -> Self {
        BusinessResponse {
            id: business.id,
            name: business.name,
            email: business.email,
            created_at: business.created_at,
            status: business.status,
        }
    }
}
