## Architecture Overview

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#9ca3af', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Frontend["Frontend (React)"]
        FE[React App<br/>Port 5000]
    end

    subgraph Gateway["API Gateway"]
        GW[Gateway<br/>Port 10000]
    end

    subgraph Services["Backend Services"]
        AUTH[Auth Service<br/>Port 50051<br/>gRPC]
        EVENT[Event Service<br/>Port 8001<br/>REST]
        CLIENT[Client Service<br/>Port 8002<br/>REST]
        EMAIL[Email Service<br/>Port 50052<br/>gRPC]
        NOTIF[Notification Service<br/>Port 8004<br/>WebSocket]
    end

    subgraph MessageQueue["Message Queue"]
        RMQ[RabbitMQ<br/>Port 5672]
    end

    subgraph Databases["Databases"]
        AUTH_DB[(PostgreSQL<br/>auth-db)]
        EVENT_DB[(PostgreSQL<br/>event-db)]
        CLIENT_DB[(MongoDB<br/>client-db)]
        SHARED_REDIS[(Redis<br/>shared-redis)]
    end

    FE -->|JSON| GW
    GW -->|gRPC| AUTH
    GW -->|gRPC| EMAIL
    GW -->|JSON| FE

    FE -->|REST| EVENT
    FE -->|REST| CLIENT
    FE -->|WebSocket| NOTIF

    EVENT -->|gRPC| AUTH
    CLIENT -->|gRPC| AUTH
    CLIENT -->|gRPC| EMAIL
    CLIENT -->|REST| EVENT
    NOTIF -->|gRPC| AUTH

    CLIENT -->|publish| RMQ
    RMQ -->|consume| EVENT
    EVENT -->|publish| RMQ
    RMQ -->|consume| CLIENT
    RMQ -->|consume| EMAIL
    RMQ -->|consume| NOTIF

    AUTH --> AUTH_DB
    AUTH --> SHARED_REDIS
    EVENT --> EVENT_DB
    CLIENT --> CLIENT_DB
    EMAIL --> SHARED_REDIS

    style RMQ fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style AUTH fill:#4dabf7,stroke:#1971c2,color:#fff
    style EVENT fill:#69db7c,stroke:#2f9e44,color:#fff
    style CLIENT fill:#ffd43b,stroke:#f59f00,color:#000
    style EMAIL fill:#da77f2,stroke:#9c36b5,color:#fff
    style NOTIF fill:#ff922b,stroke:#e8590c,color:#fff
    style GW fill:#868e96,stroke:#495057,color:#fff
    style FE fill:#20c997,stroke:#0ca678,color:#fff
```

> [!TIP]
> The entire backend is built in **Rust** using the **Axum** web framework. Each service has detailed documentation in the `doc/` folder:
>
> **Services:**
> - [Gateway](doc/gateway.md) — API Gateway with rate limiting (Port 10000)
> - [Auth Service](doc/auth-service.md) — Authentication & JWT management (Port 50051, gRPC)
> - [Event Service](doc/event-service.md) — Event, ticket & refund management (Port 8001, REST)
> - [Client Service](doc/client-service.md) — Client profiles & ticket purchases (Port 8002, REST)
> - [Email Service](doc/email-service.md) — Email delivery & verification codes (Port 50052, gRPC)
> - [Notification Service](doc/notification-service.md) — Real-time WebSocket notifications (Port 8004)
>
> **Infrastructure:**
> - [RabbitMQ](doc/rabbitmq.md) — Message broker for async communication
> - [Grafana](doc/grafana.md) — Monitoring & logging dashboard

## Quick Start

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# Full cleanup (removes database volumes)
docker-compose down -v
```

> [!NOTE]
> All databases are stored in Docker volumes for persistence. Use `docker-compose down -v` for a complete cleanup including all data.

