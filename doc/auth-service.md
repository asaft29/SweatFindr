# Auth Service

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#9ca3af', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Clients["Clients (gRPC)"]
        GW[Gateway]
        EVENT[Event Service]
        CLIENT[Client Service]
        NOTIF[Notification Service]
    end

    subgraph AuthService["Auth Service - Port 50051"]
        GRPC[gRPC Server]
        HANDLERS[Handlers]
        JWT[JWT Manager]
        BLACKLIST[Token Blacklist]
    end

    subgraph Storage["Storage"]
        PG[(PostgreSQL<br/>Users Table)]
        REDIS[(Redis<br/>Token Blacklist)]
    end

    GW -->|Authenticate<br/>ValidateToken| GRPC
    EVENT -->|ValidateToken| GRPC
    CLIENT -->|ValidateToken<br/>RegisterUser| GRPC
    NOTIF -->|ValidateToken| GRPC

    GRPC --> HANDLERS
    HANDLERS --> JWT
    HANDLERS --> BLACKLIST
    HANDLERS --> PG
    BLACKLIST --> REDIS

    style GRPC fill:#4dabf7,stroke:#1971c2,color:#fff
    style PG fill:#336791,stroke:#1d3d5c,color:#fff
    style REDIS fill:#dc382d,stroke:#a52a2a,color:#fff
```

Central authentication provider handling user management and JWT tokens via gRPC.

## How It Works

The Auth Service is the security backbone of the platform. All other services validate tokens through this service via gRPC calls.

**Authentication Flow:**
1. User submits credentials via Gateway
2. Auth Service verifies against PostgreSQL (bcrypt hashed passwords)
3. JWT token generated with user ID, email, and role
4. Token returned to client for subsequent requests

**Token Validation:**
1. Services call `ValidateToken` with Bearer token
2. Auth Service checks Redis blacklist first
3. If not blacklisted, decodes and validates JWT
4. Returns user claims (id, email, role)

**Logout:**
1. Token added to Redis blacklist with TTL matching token expiration
2. Subsequent validation requests reject blacklisted tokens

## gRPC Endpoints

| Method | Description |
|--------|-------------|
| `Authenticate` | Login with email/password, returns JWT |
| `ValidateToken` | Verify token, returns user claims |
| `DestroyToken` | Logout - adds token to blacklist |
| `RegisterUser` | Create new user account |
| `GetUserEmail` | Get email by user ID |
| `GetUserIdByEmail` | Get user ID and verification status by email |
| `UpdateRole` | Change user role |
| `MarkEmailVerified` | Mark email as verified |
| `ResetPassword` | Reset user password |
| `DeleteUser` | Delete verified user |

## Database Schema

```sql
UTILIZATORI
├─ ID              SERIAL PRIMARY KEY
├─ email           VARCHAR UNIQUE
├─ parola          VARCHAR (bcrypt hash)
├─ rol             VARCHAR (admin|owner-event|client|clients-service)
└─ email_verified  BOOLEAN
```

## Environment Variables

```bash
DATABASE_URL=postgresql://user:pass@auth-db:5432/auth
REDIS_URL=redis://auth-redis:6379
JWT_SECRET=your-secret-key
JWT_EXPIRATION=3600
```
