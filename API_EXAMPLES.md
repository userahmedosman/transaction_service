# API Usage Examples

This document provides practical examples of using the Transaction Service API.

## Prerequisites

1. Start the service: `cargo run`
2. Service runs on `http://localhost:8080`

## Example Flow

### 1. Create a Business

```bash
curl -X POST http://localhost:8080/businesses \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Acme Corporation",
    "email": "finance@acme.com"
  }'
```

Response:
```json
{
  "id": "123e4567-e89b-12d3-a456-426614174000",
  "name": "Acme Corporation",
  "email": "finance@acme.com",
  "created_at": "2025-12-19T10:00:00Z",
  "status": "active"
}
```

### 2. Create an API Key

```bash
curl -X POST http://localhost:8080/api-keys \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Production API Key",
    "rate_limit": 5000
  }'
```


Response:
```json
{
  "id": "456e7890-e89b-12d3-a456-426614174111",
  "name": "Production API Key",
  "key": "sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p",
  "key_prefix": "sk_1a2b3c4d5",
  "created_at": "2025-12-19T10:01:00Z",
  "expires_at": null,
  "rate_limit": 5000
}
```

**Important**: Save the `key` value. It won't be shown again!

### 3. Create Accounts

Create a source account:
```bash
curl -X POST http://localhost:8080/api/v1/accounts \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "account_type": "checking",
    "currency": "USD"
  }'
```

Response:
```json
{
  "id": "789e0123-e89b-12d3-a456-426614174222",
  "account_number": "ACCa1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p",
  "account_type": "checking",
  "currency": "USD",
  "balance": 0,
  "available_balance": 0,
  "created_at": "2025-12-19T10:02:00Z",
  "status": "active"
}
```

Create a destination account:
```bash
curl -X POST http://localhost:8080/api/v1/accounts \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "account_type": "savings",
    "currency": "USD"
  }'
```

### 4. Deposit Funds

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "deposit-001",
    "transaction_type": "deposit",
    "amount": 100000,
    "currency": "USD",
    "destination_account_id": "789e0123-e89b-12d3-a456-426614174222",
    "description": "Initial deposit",
    "metadata": {
      "source": "bank_transfer",
      "reference": "WIRE123456"
    }
  }'
```

Response:
```json
{
  "id": "abc12345-e89b-12d3-a456-426614174333",
  "idempotency_key": "deposit-001",
  "transaction_type": "deposit",
  "amount": 100000,
  "currency": "USD",
  "source_account_id": null,
  "destination_account_id": "789e0123-e89b-12d3-a456-426614174222",
  "status": "pending",
  "description": "Initial deposit",
  "metadata": {
    "source": "bank_transfer",
    "reference": "WIRE123456"
  },
  "created_at": "2025-12-19T10:03:00Z",
  "completed_at": null,
  "error_message": null
}
```

### 5. Check Transaction Status

```bash
curl http://localhost:8080/api/v1/transactions/abc12345-e89b-12d3-a456-426614174333 \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p"
```

### 6. Transfer Between Accounts

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "transfer-001",
    "transaction_type": "transfer",
    "amount": 50000,
    "currency": "USD",
    "source_account_id": "789e0123-e89b-12d3-a456-426614174222",
    "destination_account_id": "def45678-e89b-12d3-a456-426614174444",
    "description": "Savings transfer"
  }'
```

### 7. Check Account Balance

```bash
curl http://localhost:8080/api/v1/accounts/789e0123-e89b-12d3-a456-426614174222/balance \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p"
```

Response:
```json
{
  "account_id": "789e0123-e89b-12d3-a456-426614174222",
  "balance": 50000,
  "available_balance": 50000,
  "currency": "USD"
}
```

### 8. List All Transactions

```bash
curl "http://localhost:8080/api/v1/transactions?limit=10&offset=0" \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p"
```

### 9. Get Transaction Logs (Audit Trail)

```bash
curl http://localhost:8080/api/v1/transactions/abc12345-e89b-12d3-a456-426614174333/logs \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p"
```

Response:
```json
[
  {
    "id": 1,
    "previous_status": "pending",
    "new_status": "completed",
    "changed_at": "2025-12-19T10:03:01Z",
    "reason": "Transaction completed successfully"
  },
  {
    "id": 2,
    "previous_status": null,
    "new_status": "pending",
    "changed_at": "2025-12-19T10:03:00Z",
    "reason": "Transaction created"
  }
]
```

### 10. Set Up Webhook

```bash
curl -X POST http://localhost:8080/api/v1/webhooks \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "url": "https://your-app.com/webhooks/transactions",
    "events": ["transaction.completed", "transaction.failed"]
  }'
```

Response:
```json
{
  "id": "webhook-123",
  "url": "https://your-app.com/webhooks/transactions",
  "events": ["transaction.completed", "transaction.failed"],
  "is_active": true,
  "created_at": "2025-12-19T10:05:00Z"
}
```

## Idempotency Example

Sending the same request twice with the same idempotency key:

First request:
```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "unique-payment-123",
    "transaction_type": "transfer",
    "amount": 10000,
    "currency": "USD",
    "source_account_id": "source-id",
    "destination_account_id": "dest-id"
  }'
```

Second request (same idempotency_key):
```bash
# Same request as above
```

Result: Both requests return the same transaction. No duplicate is created!

## Error Handling Examples

### Insufficient Funds

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Authorization: Bearer sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p" \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "transfer-overdraft",
    "transaction_type": "withdrawal",
    "amount": 999999999,
    "currency": "USD",
    "source_account_id": "789e0123-e89b-12d3-a456-426614174222"
  }'
```

Response (after processing):
```json
{
  "id": "...",
  "status": "failed",
  "error_message": "Insufficient funds",
  ...
}
```

### Invalid API Key

```bash
curl http://localhost:8080/api/v1/accounts \
  -H "Authorization: Bearer invalid_key"
```

Response:
```json
{
  "error": "Invalid API key"
}
```

### Rate Limit Exceeded

After making too many requests:
```json
{
  "error": "Rate limit exceeded"
}
```

## Testing Webhook Delivery

You can use a service like webhook.site to test webhook delivery:

1. Go to https://webhook.site
2. Copy your unique URL
3. Create a webhook endpoint with that URL
4. Create a transaction
5. Check webhook.site for the delivery

Webhook payload example:
```json
{
  "event_type": "transaction.completed",
  "transaction": {
    "id": "abc12345-e89b-12d3-a456-426614174333",
    "idempotency_key": "deposit-001",
    "transaction_type": "deposit",
    "amount": 100000,
    "currency": "USD",
    "status": "completed",
    "created_at": "2025-12-19T10:03:00Z",
    "completed_at": "2025-12-19T10:03:01Z"
  }
}
```

## Python Client Example

```python
import requests
import uuid

BASE_URL = "http://localhost:8080"
API_KEY = "sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p"

headers = {
    "Authorization": f"Bearer {API_KEY}",
    "Content-Type": "application/json"
}

# Create account
response = requests.post(
    f"{BASE_URL}/api/v1/accounts",
    headers=headers,
    json={
        "account_type": "checking",
        "currency": "USD"
    }
)
account = response.json()
print(f"Created account: {account['id']}")

# Create transaction
response = requests.post(
    f"{BASE_URL}/api/v1/transactions",
    headers=headers,
    json={
        "idempotency_key": f"payment-{uuid.uuid4()}",
        "transaction_type": "deposit",
        "amount": 50000,
        "currency": "USD",
        "destination_account_id": account["id"],
        "description": "Initial deposit"
    }
)
transaction = response.json()
print(f"Created transaction: {transaction['id']}")
```

## JavaScript/Node.js Client Example

```javascript
const axios = require('axios');
const { v4: uuidv4 } = require('uuid');

const BASE_URL = 'http://localhost:8080';
const API_KEY = 'sk_1a2b3c4d5e6f7g8h9i0j1k2l3m4n5o6p';

const headers = {
  'Authorization': `Bearer ${API_KEY}`,
  'Content-Type': 'application/json'
};

async function createTransaction() {
  try {
    const response = await axios.post(
      `${BASE_URL}/api/v1/transactions`,
      {
        idempotency_key: `payment-${uuidv4()}`,
        transaction_type: 'deposit',
        amount: 50000,
        currency: 'USD',
        destination_account_id: 'account-id-here',
        description: 'Initial deposit'
      },
      { headers }
    );
    
    console.log('Transaction created:', response.data);
  } catch (error) {
    console.error('Error:', error.response?.data || error.message);
  }
}

createTransaction();
```

## Advanced Usage

### Pagination

```bash
# Get first page
curl "http://localhost:8080/api/v1/transactions?limit=20&offset=0" \
  -H "Authorization: Bearer sk_..."

# Get second page
curl "http://localhost:8080/api/v1/transactions?limit=20&offset=20" \
  -H "Authorization: Bearer sk_..."
```

### Metadata for Complex Use Cases

```bash
curl -X POST http://localhost:8080/api/v1/transactions \
  -H "Authorization: Bearer sk_..." \
  -H "Content-Type: application/json" \
  -d '{
    "idempotency_key": "invoice-payment-123",
    "transaction_type": "transfer",
    "amount": 50000,
    "currency": "USD",
    "source_account_id": "...",
    "destination_account_id": "...",
    "description": "Invoice payment",
    "metadata": {
      "invoice_id": "INV-2025-001",
      "customer_id": "CUST-456",
      "payment_method": "bank_transfer",
      "tax_amount": 5000,
      "processing_fee": 500,
      "custom_field": "any value"
    }
  }'
```

## Health Check

```bash
curl http://localhost:8080/health
```

Response:
```json
{
  "status": "ok",
  "database": "healthy",
  "service": "transaction-service",
  "version": "0.1.0"
}
```

## Tips

1. **Always use unique idempotency keys** for new transactions
2. **Store your API key securely** - never commit it to version control
3. **Use appropriate HTTP methods**: POST for creation, GET for retrieval
4. **Handle rate limiting** in your client with exponential backoff
5. **Monitor webhook deliveries** for failed attempts
6. **Use metadata** to store additional context about transactions
7. **Check transaction status** asynchronously for large amounts
