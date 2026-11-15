use crate::models::{
    error::{AppError, Result},
    Account, CreateTransactionRequest, Transaction, TransactionLog,
};
use chrono::Utc;
use serde_json::json;
use sqlx::{PgPool, Postgres, Transaction as SqlxTransaction};
use uuid::Uuid;

pub struct TransactionService {
    pool: PgPool,
}

impl TransactionService {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }

    pub async fn create_transaction(
        &self,
        business_id: Uuid,
        request: CreateTransactionRequest,
    ) -> Result<Transaction> {
        // Validate transaction type
        match request.transaction_type.as_str() {
            "transfer" | "deposit" | "withdrawal" => {}
            _ => {
                return Err(AppError::BadRequest(
                    "Invalid transaction type".to_string(),
                ))
            }
        }

        // Validate account ownership
        if let Some(source_id) = request.source_account_id {
            self.verify_account_ownership(business_id, source_id).await?;
        }
        if let Some(dest_id) = request.destination_account_id {
            self.verify_account_ownership(business_id, dest_id).await?;
        }

        // Start database transaction
        let mut tx = self.pool.begin().await?;

        // Check for duplicate idempotency key
        let existing = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions 
            WHERE business_id = $1 AND idempotency_key = $2
            "#,
        )
        .bind(business_id)
        .bind(&request.idempotency_key)
        .fetch_optional(&mut *tx)
        .await?;

        if let Some(existing_tx) = existing {
            return Ok(existing_tx);
        }

        // Create transaction record
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"
            INSERT INTO transactions 
            (idempotency_key, business_id, transaction_type, amount, currency,
             source_account_id, destination_account_id, status, description, metadata)
            VALUES ($1, $2, $3, $4, $5, $6, $7, 'pending', $8, $9)
            RETURNING *
            "#,
        )
        .bind(&request.idempotency_key)
        .bind(business_id)
        .bind(&request.transaction_type)
        .bind(request.amount)
        .bind(&request.currency)
        .bind(request.source_account_id)
        .bind(request.destination_account_id)
        .bind(&request.description)
        .bind(&request.metadata)
        .fetch_one(&mut *tx)
        .await?;

        // Log transaction creation
        self.log_transaction_status(
            &mut tx,
            transaction.id,
            None,
            "pending",
            Some("Transaction created"),
        )
        .await?;

        tx.commit().await?;

        // Process transaction asynchronously
        let pool = self.pool.clone();
        let tx_id = transaction.id;
        tokio::spawn(async move {
            let service = TransactionService::new(pool);
            if let Err(e) = service.process_transaction(tx_id).await {
                tracing::error!("Failed to process transaction {}: {:?}", tx_id, e);
            }
        });

        Ok(transaction)
    }

    async fn process_transaction(&self, transaction_id: Uuid) -> Result<()> {
        let mut tx = self.pool.begin().await?;

        // Get transaction
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"SELECT * FROM transactions WHERE id = $1 FOR UPDATE"#,
        )
        .bind(transaction_id)
        .fetch_one(&mut *tx)
        .await?;

        if transaction.status != "pending" {
            return Ok(()); // Already processed
        }

        // Process based on transaction type
        let result = match transaction.transaction_type.as_str() {
            "transfer" => self.process_transfer(&mut tx, &transaction).await,
            "deposit" => self.process_deposit(&mut tx, &transaction).await,
            "withdrawal" => self.process_withdrawal(&mut tx, &transaction).await,
            _ => Err(AppError::BadRequest("Invalid transaction type".to_string())),
        };

        match result {
            Ok(_) => {
                // Update transaction status to completed
                sqlx::query(
                    r#"
                    UPDATE transactions 
                    SET status = 'completed', completed_at = NOW()
                    WHERE id = $1
                    "#,
                )
                .bind(transaction_id)
                .execute(&mut *tx)
                .await?;

                self.log_transaction_status(
                    &mut tx,
                    transaction_id,
                    Some("pending"),
                    "completed",
                    Some("Transaction completed successfully"),
                )
                .await?;
            }
            Err(e) => {
                // Update transaction status to failed
                sqlx::query(
                    r#"
                    UPDATE transactions 
                    SET status = 'failed', error_message = $1, completed_at = NOW()
                    WHERE id = $2
                    "#,
                )
                .bind(e.to_string())
                .bind(transaction_id)
                .execute(&mut *tx)
                .await?;

                self.log_transaction_status(
                    &mut tx,
                    transaction_id,
                    Some("pending"),
                    "failed",
                    Some(&e.to_string()),
                )
                .await?;
            }
        }

        tx.commit().await?;
        Ok(())
    }

    async fn process_transfer(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        transaction: &Transaction,
    ) -> Result<()> {
        let source_id = transaction
            .source_account_id
            .ok_or_else(|| AppError::BadRequest("Source account required".to_string()))?;
        let dest_id = transaction
            .destination_account_id
            .ok_or_else(|| AppError::BadRequest("Destination account required".to_string()))?;

        // Lock accounts for update
        let source = sqlx::query_as::<_, Account>(
            r#"SELECT * FROM accounts WHERE id = $1 FOR UPDATE"#,
        )
        .bind(source_id)
        .fetch_one(&mut **tx)
        .await?;

        let dest = sqlx::query_as::<_, Account>(
            r#"SELECT * FROM accounts WHERE id = $1 FOR UPDATE"#,
        )
        .bind(dest_id)
        .fetch_one(&mut **tx)
        .await?;

        // Verify currency match
        if source.currency != dest.currency || source.currency != transaction.currency {
            return Err(AppError::BadRequest("Currency mismatch".to_string()));
        }

        // Check sufficient balance
        if source.available_balance < transaction.amount {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        // Update balances
        sqlx::query(
            r#"
            UPDATE accounts 
            SET balance = balance - $1, 
                available_balance = available_balance - $1,
                version = version + 1
            WHERE id = $2 AND version = $3
            "#,
        )
        .bind(transaction.amount)
        .bind(source_id)
        .bind(source.version)
        .execute(&mut **tx)
        .await?;

        sqlx::query(
            r#"
            UPDATE accounts 
            SET balance = balance + $1, 
                available_balance = available_balance + $1,
                version = version + 1
            WHERE id = $2 AND version = $3
            "#,
        )
        .bind(transaction.amount)
        .bind(dest_id)
        .bind(dest.version)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn process_deposit(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        transaction: &Transaction,
    ) -> Result<()> {
        let dest_id = transaction
            .destination_account_id
            .ok_or_else(|| AppError::BadRequest("Destination account required".to_string()))?;

        let account = sqlx::query_as::<_, Account>(
            r#"SELECT * FROM accounts WHERE id = $1 FOR UPDATE"#,
        )
        .bind(dest_id)
        .fetch_one(&mut **tx)
        .await?;

        if account.currency != transaction.currency {
            return Err(AppError::BadRequest("Currency mismatch".to_string()));
        }

        sqlx::query(
            r#"
            UPDATE accounts 
            SET balance = balance + $1, 
                available_balance = available_balance + $1,
                version = version + 1
            WHERE id = $2
            "#,
        )
        .bind(transaction.amount)
        .bind(dest_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn process_withdrawal(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        transaction: &Transaction,
    ) -> Result<()> {
        let source_id = transaction
            .source_account_id
            .ok_or_else(|| AppError::BadRequest("Source account required".to_string()))?;

        let account = sqlx::query_as::<_, Account>(
            r#"SELECT * FROM accounts WHERE id = $1 FOR UPDATE"#,
        )
        .bind(source_id)
        .fetch_one(&mut **tx)
        .await?;

        if account.currency != transaction.currency {
            return Err(AppError::BadRequest("Currency mismatch".to_string()));
        }

        if account.available_balance < transaction.amount {
            return Err(AppError::BadRequest("Insufficient funds".to_string()));
        }

        sqlx::query(
            r#"
            UPDATE accounts 
            SET balance = balance - $1, 
                available_balance = available_balance - $1,
                version = version + 1
            WHERE id = $2
            "#,
        )
        .bind(transaction.amount)
        .bind(source_id)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    async fn verify_account_ownership(&self, business_id: Uuid, account_id: Uuid) -> Result<()> {
        let exists = sqlx::query_as::<_, (bool,)>(
            r#"SELECT EXISTS(SELECT 1 FROM accounts WHERE id = $1 AND business_id = $2)"#,
        )
        .bind(account_id)
        .bind(business_id)
        .fetch_one(&self.pool)
        .await?;

        if !exists.0 {
            return Err(AppError::NotFound("Account not found".to_string()));
        }

        Ok(())
    }

    async fn log_transaction_status(
        &self,
        tx: &mut SqlxTransaction<'_, Postgres>,
        transaction_id: Uuid,
        previous_status: Option<&str>,
        new_status: &str,
        reason: Option<&str>,
    ) -> Result<()> {
        sqlx::query(
            r#"
            INSERT INTO transaction_logs 
            (transaction_id, previous_status, new_status, reason)
            VALUES ($1, $2, $3, $4)
            "#,
        )
        .bind(transaction_id)
        .bind(previous_status)
        .bind(new_status)
        .bind(reason)
        .execute(&mut **tx)
        .await?;

        Ok(())
    }

    pub async fn get_transaction(&self, business_id: Uuid, id: Uuid) -> Result<Transaction> {
        let transaction = sqlx::query_as::<_, Transaction>(
            r#"SELECT * FROM transactions WHERE id = $1 AND business_id = $2"#,
        )
        .bind(id)
        .bind(business_id)
        .fetch_optional(&self.pool)
        .await?
        .ok_or_else(|| AppError::NotFound("Transaction not found".to_string()))?;

        Ok(transaction)
    }

    pub async fn list_transactions(
        &self,
        business_id: Uuid,
        limit: i64,
        offset: i64,
    ) -> Result<Vec<Transaction>> {
        let transactions = sqlx::query_as::<_, Transaction>(
            r#"
            SELECT * FROM transactions 
            WHERE business_id = $1
            ORDER BY created_at DESC
            LIMIT $2 OFFSET $3
            "#,
        )
        .bind(business_id)
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(transactions)
    }

    pub async fn get_transaction_logs(
        &self,
        business_id: Uuid,
        transaction_id: Uuid,
    ) -> Result<Vec<TransactionLog>> {
        // Verify transaction belongs to business
        self.get_transaction(business_id, transaction_id).await?;

        let logs = sqlx::query_as::<_, TransactionLog>(
            r#"
            SELECT * FROM transaction_logs 
            WHERE transaction_id = $1
            ORDER BY changed_at DESC
            "#,
        )
        .bind(transaction_id)
        .fetch_all(&self.pool)
        .await?;

        Ok(logs)
    }
}
