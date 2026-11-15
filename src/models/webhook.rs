use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;
use sqlx::FromRow;
use uuid::Uuid;
use validator::Validate;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookEndpoint {
    pub id: Uuid,
    pub business_id: Uuid,
    pub url: String,
    pub secret: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub max_retries: i32,
    pub retry_delay_seconds: i32,
}

#[derive(Debug, Deserialize, Validate)]
pub struct CreateWebhookEndpointRequest {
    #[validate(url)]
    pub url: String,
    pub events: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WebhookEndpointResponse {
    pub id: Uuid,
    pub url: String,
    pub events: Vec<String>,
    pub is_active: bool,
    pub created_at: DateTime<Utc>,
}

impl From<WebhookEndpoint> for WebhookEndpointResponse {
    fn from(endpoint: WebhookEndpoint) -> Self {
        WebhookEndpointResponse {
            id: endpoint.id,
            url: endpoint.url,
            events: endpoint.events,
            is_active: endpoint.is_active,
            created_at: endpoint.created_at,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WebhookDelivery {
    pub id: Uuid,
    pub webhook_endpoint_id: Uuid,
    pub transaction_id: Uuid,
    pub event_type: String,
    pub payload: JsonValue,
    pub status: String,
    pub attempt_count: i32,
    pub next_retry_at: Option<DateTime<Utc>>,
    pub last_attempt_at: Option<DateTime<Utc>>,
    pub response_status_code: Option<i32>,
    pub response_body: Option<String>,
    pub error_message: Option<String>,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Serialize)]
pub struct WebhookDeliveryResponse {
    pub id: Uuid,
    pub event_type: String,
    pub status: String,
    pub attempt_count: i32,
    pub created_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
}

impl From<WebhookDelivery> for WebhookDeliveryResponse {
    fn from(delivery: WebhookDelivery) -> Self {
        WebhookDeliveryResponse {
            id: delivery.id,
            event_type: delivery.event_type,
            status: delivery.status,
            attempt_count: delivery.attempt_count,
            created_at: delivery.created_at,
            completed_at: delivery.completed_at,
        }
    }
}
