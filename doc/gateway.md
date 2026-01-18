# Gateway

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#9ca3af', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Clients["Clients"]
        FE[Frontend]
    end

    subgraph Gateway["Gateway - Port 10000"]
        REST["REST API - Axum"]
        RATE["Rate Limiter - Governor"]
        CORS[CORS Middleware]
        HANDLERS[Handlers]
    end

    subgraph Backend["Backend Services"]
        AUTH["Auth Service - gRPC :50051"]
        EMAIL["Email Service - gRPC :50052"]
    end

    FE --> REST

    REST --> CORS
    CORS --> RATE
    RATE --> HANDLERS
    HANDLERS --> AUTH
    HANDLERS --> EMAIL

    style REST fill:#868e96,stroke:#495057,color:#fff
    style AUTH fill:#4dabf7,stroke:#1971c2,color:#fff
    style EMAIL fill:#da77f2,stroke:#9c36b5,color:#fff
    style RATE fill:#ffd43b,stroke:#f59f00,color:#000
```

API Gateway providing rate limiting and proxying to auth/email gRPC services.

## How It Works

The Gateway is the main entry point for authentication and email operations, translating REST requests to gRPC calls.

**Request Flow:**
1. Frontend sends REST request
2. CORS middleware validates origin
3. Rate limiter checks request quota
4. Handler translates to gRPC call
5. Response returned to frontend

**Rate Limiting:**
- Uses Token Bucket algorithm (rather simple in comparison to other algorithms)
- Returns 429 Too Many Requests with Retry-After header

## REST Endpoints

### Authentication
| Method | Endpoint | Proxies To |
|--------|----------|------------|
| `POST` | `/auth/login` | Auth.Authenticate |
| `POST` | `/auth/logout` | Auth.DestroyToken |
| `POST` | `/auth/validate` | Auth.ValidateToken |

### Email
| Method | Endpoint | Proxies To |
|--------|----------|------------|
| `POST` | `/email/send-verification` | Email.SendVerificationEmail |
| `POST` | `/email/verify` | Email.VerifyCode |
| `POST` | `/email/resend` | Email.ResendVerificationCode |
| `POST` | `/email/forgot-password` | Email.SendPasswordResetEmail |
| `POST` | `/email/reset-password` | Email.VerifyPasswordResetCode |

## Rate Limit Response

```http
HTTP/1.1 429 Too Many Requests
Retry-After: 30

{
  "error": "Too many requests",
  "retry_after": 30
}
```
