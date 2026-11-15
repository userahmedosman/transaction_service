use crate::models::{
    error::{AppError, Result},
    Transaction, WebhookDelivery, WebhookEndpoint,
};
use chrono::{Duration, Utc};
use serde_json::json;
use sqlx::PgPool;
use uuid::Uuid;

pub struct WebhookService {
    pool: PgPool,
    client: reqwest::Client,
}

impl WebhookService {
    pub fn new(pool: PgPool) -> Self {
        Self {
            pool,
            client: reqwest::Client::new(),
        }
    }

    pub async fn trigger_webhook(&self, transaction: &Transaction, event_type: &str) -> Result<()> {
        // Get active webhook endpoints for this business
        let endpoints = sqlx::query_as::<_, WebhookEndpoint>(
            r#"
            SELECT * FROM webhook_endpoints
            WHERE business_id = $1 AND is_active = true
            "#,
        )
        .bind(transaction.business_id)
        .fetch_all(&self.pool)
        .await?;

        for endpoint in endpoints {
            // Check if endpoint is subscribed to this event
            if !endpoint.events.contains(&event_type.to_string()) {
                continue;
            }

            // Create webhook delivery record
            let payload = json!({
                "event_type": event_type,
                "transaction": {
                    "id": transaction.id,
                    "idempotency_key": transaction.idempotency_key,
                    "transaction_type": transaction.transaction_type,
                    "amount": transaction.amount,
                    "currency": transaction.currency,
                    "status": transaction.status,
                    "created_at": transaction.created_at,
                    "completed_at": transaction.completed_at,
                }
            });

            let delivery = sqlx::query_as::<_, WebhookDelivery>(
                r#"
                INSERT INTO webhook_deliveries 
                (webhook_endpoint_id, transaction_id, event_type, payload, status)
                VALUES ($1, $2, $3, $4, 'pending')
                RETURNING *
                "#,
            )
            .bind(endpoint.id)
            .bind(transaction.id)
            .bind(event_type)
            .bind(&payload)
            .fetch_one(&self.pool)
            .await?;

            // Schedule delivery
            let pool = self.pool.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let service = WebhookService { pool, client };
                if let Err(e) = service.deliver_webhook(delivery.id).await {
                    tracing::error!("Failed to deliver webhook {}: {:?}", delivery.id, e);
                }
            });
        }

        Ok(())
    }

    pub async fn deliver_webhook(&self, delivery_id: Uuid) -> Result<()> {
        let mut delivery = sqlx::query_as::<_, WebhookDelivery>(
            r#"SELECT * FROM webhook_deliveries WHERE id = $1"#,
        )
        .bind(delivery_id)
        .fetch_one(&self.pool)
        .await?;

        let endpoint = sqlx::query_as::<_, WebhookEndpoint>(
            r#"SELECT * FROM webhook_endpoints WHERE id = $1"#,
        )
        .bind(delivery.webhook_endpoint_id)
        .fetch_one(&self.pool)
        .await?;

        // Attempt delivery
        let result = self
            .client
            .post(&endpoint.url)
            .header("Content-Type", "application/json")
            .header("X-Webhook-Secret", &endpoint.secret)
            .json(&delivery.payload)
            .send()
            .await;

        delivery.attempt_count += 1;
        delivery.last_attempt_at = Some(Utc::now());

        match result {
            Ok(response) => {
                delivery.response_status_code = Some(response.status().as_u16() as i32);
                
                if response.status().is_success() {
                    delivery.status = "completed".to_string();
                    delivery.completed_at = Some(Utc::now());
                } else {
                    let body = response.text().await.unwrap_or_default();
                    delivery.response_body = Some(body.clone());
                    
                    if delivery.attempt_count >= endpoint.max_retries {
                        delivery.status = "failed".to_string();
                        delivery.completed_at = Some(Utc::now());
                    } else {
                        delivery.status = "pending".to_string();
                        delivery.next_retry_at = Some(
                            Utc::now() + Duration::seconds(endpoint.retry_delay_seconds as i64)
                        );
                    }
                }
            }
            Err(e) => {
                delivery.error_message = Some(e.to_string());
                
                if delivery.attempt_count >= endpoint.max_retries {
                    delivery.status = "failed".to_string();
                    delivery.completed_at = Some(Utc::now());
                } else {
                    delivery.status = "pending".to_string();
                    delivery.next_retry_at = Some(
                        Utc::now() + Duration::seconds(endpoint.retry_delay_seconds as i64)
                    );
                }
            }
        }

        // Update delivery record
        sqlx::query(
            r#"
            UPDATE webhook_deliveries
            SET status = $1, attempt_count = $2, last_attempt_at = $3,
                next_retry_at = $4, response_status_code = $5, response_body = $6,
                error_message = $7, completed_at = $8
            WHERE id = $9
            "#,
        )
        .bind(&delivery.status)
        .bind(delivery.attempt_count)
        .bind(delivery.last_attempt_at)
        .bind(delivery.next_retry_at)
        .bind(delivery.response_status_code)
        .bind(&delivery.response_body)
        .bind(&delivery.error_message)
        .bind(delivery.completed_at)
        .bind(delivery_id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn retry_failed_webhooks(&self) -> Result<()> {
        let pending_deliveries = sqlx::query_as::<_, WebhookDelivery>(
            r#"
            SELECT * FROM webhook_deliveries
            WHERE status = 'pending' 
            AND next_retry_at <= NOW()
            LIMIT 100
            "#,
        )
        .fetch_all(&self.pool)
        .await?;

        for delivery in pending_deliveries {
            let pool = self.pool.clone();
            let client = self.client.clone();
            tokio::spawn(async move {
                let service = WebhookService { pool, client };
                if let Err(e) = service.deliver_webhook(delivery.id).await {
                    tracing::error!("Failed to retry webhook {}: {:?}", delivery.id, e);
                }
            });
        }

        Ok(())
    }
}
