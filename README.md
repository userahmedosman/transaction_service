# Transaction Service

A robust, production-ready transaction service built with Rust, Axum, and SQLx. This service provides a complete solution for managing financial transactions, accounts, and webhooks with built-in idempotency, rate limiting, and security features.

## Features

### Core Functionality
- **Transaction Management**: Support for transfers, deposits, and withdrawals
- **Account Management**: Multi-currency account support with balance tracking
- **Idempotency**: Built-in idempotency key handling to prevent duplicate transactions
- **Audit Trail**: Complete transaction logging with status changes
- **Webhook Support**: Event-driven webhooks with automatic retries
- **API Key Authentication**: Secure API key-based authentication with hashing
- **Rate Limiting**: Per-API-key rate limiting with token bucket algorithm

### Architecture Highlights
- **Database Transactions**: ACID-compliant with optimistic locking
- **Async Processing**: Non-blocking transaction processing with Tokio
- **RESTful API**: Clean REST API design with comprehensive error handling
- **Security**: Argon2 password hashing, SQL injection prevention
- **Observability**: Built-in logging with tracing

## Technology Stack

- **Language**: Rust (2021 Edition)
- **Web Framework**: Axum 0.7
- **Database**: PostgreSQL with SQLx
- **Async Runtime**: Tokio
- **Authentication**: API Key with Argon2 hashing
- **Rate Limiting**: Custom token bucket implementation
- **Validation**: Validator crate
- **Logging**: Tracing + Tracing-subscriber

## Project Structure

```
transaction-service/
├── src/
│   ├── handlers/          # HTTP request handlers
│   │   ├── account.rs     # Account endpoints
│   │   ├── business.rs    # Business and API key endpoints
│   │   ├── transaction.rs # Transaction endpoints
│   │   ├── webhook.rs     # Webhook endpoints
│   │   └── health.rs      # Health check endpoint
│   ├── middleware/        # HTTP middleware
│   │   ├── auth.rs        # API key authentication
│   │   └── rate_limit.rs  # Rate limiting
│   ├── models/            # Data models
│   │   ├── account.rs     # Account models
│   │   ├── api_key.rs     # API key models
│   │   ├── business.rs    # Business models
│   │   ├── transaction.rs # Transaction models
│   │   ├── webhook.rs     # Webhook models
│   │   └── error.rs       # Error types
│   ├── services/          # Business logic
│   │   ├── transaction_service.rs
│   │   └── webhook_service.rs
│   ├── utils/             # Utilities
│   │   ├── api_key.rs     # API key generation
│   │   └── db.rs          # Database connection
│   └── main.rs            # Application entry point
├── migrations/            # Database migrations
│   └── 001_initial_schema.sql
├── Cargo.toml            # Dependencies
└── .env.example          # Environment variables template
```

## Database Schema

The service uses PostgreSQL with the following main tables:

- **businesses**: Business accounts
- **accounts**: Financial accounts with balance tracking
- **api_keys**: API authentication keys
- **transactions**: Transaction records with idempotency
- **transaction_logs**: Audit trail for transactions
- **webhook_endpoints**: Webhook configurations
- **webhook_deliveries**: Webhook delivery tracking
- **rate_limit_buckets**: Rate limiting state
- **idempotency_keys**: Idempotency key tracking

## Getting Started

### Installation

1. **Clone the repository**
```bash
git clone https://github.com/divyeshkakadiya-inventyv/transaction-service.git
cd transaction-service
```
### 2. Clone and Setup Project

```bash
# Navigate to project directory
cd transaction-service

# Copy environment variables
cp .env.example .env

# Edit .env with your database credentials
# nano .env or vim .env
```

Update `.env`:

```
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/transaction_service
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info,transaction_service=debug
```

### 3 Run Postgres

```bash
# For postgres
docker compose up -d
docker ps
```

### 4 Run Migrations

```bash

# Apply database schema
sudo apt-get update
sudo apt-get install build-essential pkg-config libssl-dev
cargo install sqlx-cli --no-default-features --features postgres
sqlx --version
export DATABASE_URL=postgresql://postgres:postgres@localhost:5432/transaction_service
sqlx migrate run

docker exec -it transaction-db psql -U postgres -d transaction_service

```

### 5. Build and Run

```bash
# Build the project (this will take a few minutes the first time)
cargo build --release

# Run the service
cargo run --release
```

### 6. Testing with Postman

For easy API testing, we provide a ready-to-use Postman collection:

1. **Import Postman Collection**
```bash
   # The collection file is in the project root
   postman_collection.json
```

2. **Steps to Import:**
   - Open Postman
   - Click **Import** button (top-left)
   - Select **File** tab
   - Choose `postman_collection.json` from the project directory
   - Click **Import**

3. **Configure Environment (Optional):**
   - Set `base_url` variable to `http://0.0.0.0:8080`
   - Set `api_key` variable after creating one via the API

4. **Start Testing:**
   - The collection includes all endpoints with example requests
   - Requests are organized by feature (Business, Accounts, Transactions, Webhooks)
   - Each request has pre-filled example data

**Note:** The Postman collection includes authentication setup, so you can test protected endpoints easily after generating an API key.

### 7. API Examples Reference

For detailed API usage examples and code snippets, refer to:
```bash
API_EXAMPLES.md
```

This file contains:
- Complete request/response examples for all endpoints
- cURL commands for command-line testing
- Code examples in multiple languages
- Common use cases and workflows
- Error handling examples
- Best practices for integration

**Quick Example from API_EXAMPLES.md:**
```bash
# Create a business
curl -X POST http://localhost:8080/businesses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "My Company",
    "email": "contact@example.com"
  }'

# Create an API key
curl -X POST http://localhost:8080/api-keys \
  -H "Content-Type: application/json" \
  -d '{
    "business_id": "your-business-id",
    "name": "Production Key",
    "rate_limit": 1000
  }'
```

The service will start on `http://0.0.0.0:8080`

## API Documentation

### Authentication

All protected endpoints require an API key in the Authorization header:
```
Authorization: Bearer sk_yourapikey
```

### Public Endpoints

#### Create Business
```http
POST /businesses
Content-Type: application/json

{
  "name": "My Company",
  "email": "contact@example.com"
}
```

#### Create API Key
```http
POST /api-keys
Content-Type: application/json

{
  "name": "Production Key",
  "rate_limit": 1000,
  "expires_at": "2026-12-31T23:59:59Z"
}
```

### Protected Endpoints

#### Create Account
```http
POST /api/v1/accounts
Authorization: Bearer sk_yourapikey
Content-Type: application/json

{
  "account_type": "checking",
  "currency": "USD"
}
```

#### Create Transaction
```http
POST /api/v1/transactions
Authorization: Bearer sk_yourapikey
Content-Type: application/json

{
  "idempotency_key": "unique-key-123",
  "transaction_type": "transfer",
  "amount": 10000,
  "currency": "USD",
  "source_account_id": "uuid-of-source",
  "destination_account_id": "uuid-of-destination",
  "description": "Payment for invoice #123",
  "metadata": {
    "invoice_id": "INV-123"
  }
}
```

#### Get Transaction
```http
GET /api/v1/transactions/{transaction_id}
Authorization: Bearer sk_yourapikey
```

#### List Transactions
```http
GET /api/v1/transactions?limit=50&offset=0
Authorization: Bearer sk_yourapikey
```

#### Get Transaction Logs
```http
GET /api/v1/transactions/{transaction_id}/logs
Authorization: Bearer sk_yourapikey
```

#### Create Webhook Endpoint
```http
POST /api/v1/webhooks
Authorization: Bearer sk_yourapikey
Content-Type: application/json

{
  "url": "https://your-domain.com/webhooks/transactions",
  "events": ["transaction.completed", "transaction.failed"]
}
```

#### Get Account Balance
```http
GET /api/v1/accounts/{account_id}/balance
Authorization: Bearer sk_yourapikey
```

### Health Check
```http
GET /health
```

## Key Features Explained

### 1. Idempotency

Every transaction creation requires an `idempotency_key`. If the same key is used multiple times, the API returns the original transaction instead of creating a duplicate.

```rust
// Example: Safe retry logic
let result = create_transaction(CreateTransactionRequest {
    idempotency_key: "payment-123",  // Same key on retry
    // ... other fields
}).await?;
```

### 2. Transaction Processing

Transactions are processed asynchronously:
1. Transaction record is created with status "pending"
2. Request returns immediately
3. Background task processes the actual account updates
4. Status updates to "completed" or "failed"
5. Webhooks are triggered on status change

### 3. Account Locking

Uses optimistic locking with version numbers to prevent race conditions:
```sql
UPDATE accounts 
SET balance = balance + $1, version = version + 1
WHERE id = $2 AND version = $3
```

### 4. Rate Limiting

Token bucket algorithm with hourly refills:
- Each API key has a rate limit (default: 1000/hour)
- Tokens are consumed per request
- Tokens refill hourly
- Requests fail with 429 when limit exceeded

### 5. Webhook Retries

Failed webhook deliveries are automatically retried:
- Maximum 3 attempts (configurable)
- 60-second delay between retries (configurable)
- Exponential backoff can be added
- Full delivery tracking and logs

### Environment Variables

```bash
DATABASE_URL=postgresql://postgres:postgres@localhost:5432/transaction_service
SERVER_HOST=0.0.0.0
SERVER_PORT=8080
RUST_LOG=info,transaction_service=debug
```

## License

MIT License - see LICENSE file for details