# Event Service

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#9ca3af', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Clients["Clients"]
        FE[Frontend]
    end

    subgraph EventService["Event Service - Port 8001"]
        REST[REST API<br/>Axum]
        HANDLERS[Handlers]
        REPOS[Repositories]
        CONSUMER[Refund Consumer]
        PUBLISHER[Message Publisher]
    end

    subgraph External["External Services"]
        AUTH[Auth Service<br/>gRPC]
        RMQ[RabbitMQ]
    end

    subgraph Storage["Storage"]
        PG[(PostgreSQL<br/>Events DB)]
    end

    subgraph Upstream["Upstream Publisher"]
        CLIENT3[Client Service]
    end

    subgraph Downstream["Downstream Consumers"]
        EMAIL[Email Service]
        CLIENT2[Client Service]
        NOTIF[Notification Service]
    end

    FE --> REST

    REST --> HANDLERS
    HANDLERS --> AUTH
    HANDLERS --> REPOS
    HANDLERS --> PUBLISHER
    REPOS --> PG

    RMQ -->|refund.requested| CONSUMER
    CONSUMER --> REPOS
    CONSUMER --> PUBLISHER
    PUBLISHER -->|refund.resolved<br/>ws.broadcast| RMQ

    CLIENT3 -.->|refund.requested| RMQ
    RMQ -.->|refund.resolved| EMAIL
    RMQ -.->|refund.resolved| CLIENT2
    RMQ -.->|ws.broadcast| NOTIF

    style REST fill:#69db7c,stroke:#2f9e44,color:#fff
    style RMQ fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style PG fill:#336791,stroke:#1d3d5c,color:#fff
    style AUTH fill:#4dabf7,stroke:#1971c2,color:#fff
    style EMAIL fill:#da77f2,stroke:#9c36b5,color:#fff
    style CLIENT2 fill:#ffd43b,stroke:#f59f00,color:#000
    style CLIENT3 fill:#ffd43b,stroke:#f59f00,color:#000
    style NOTIF fill:#ff922b,stroke:#e8590c,color:#fff
```

Event, ticket, and refund management service with REST API and async message processing.

## How It Works

The *event-service* manages the core business entities: events, event packages, tickets, and refund requests.

**Event Management:**
- CRUD operations for events and event packages
- Event owners can create/edit their own events

**Ticket System:**
- Tickets are created via a certain endpoint
- *client-service* calls this endpoint when users purchase tickets
- Returns 201 Created for new tickets, 204 No Content for updates

**Refund Processing (Async via RabbitMQ):**
1. Consumes `refund.requested` messages from *client-service*
2. Creates refund request record in PostgreSQL
3. Publishes `ws.broadcast` to notify event owner via WebSocket
4. When owner approves/rejects, publishes `refund.resolved` to notify *client-service* and *email-service*


## RabbitMQ Integration

**Consumes:**
- Queue: `refund.requested.queue`
- Routing key: `refund.requested`

**Publishes:**
- Routing key: `refund.resolved` (to Email and Client services)
- Routing key: `ws.broadcast` (to Notification service)

## Database Schema

```sql
EVENIMENTE (id, id_owner, nume, locatie, descriere, numarLocuri)
PACHETE (id, id_owner, nume, locatie, descriere, numarLocuri)
JOIN_PE (PachetID, EvenimentID)
BILETE (cod, PachetID, EvenimentID)
REFUND_REQUESTS (id, ticket_cod, requester_id, status, reason, ...)
```

## Environment Variables

```bash
DATABASE_URL=postgres://user:pass@event-db:5432/eventsdb
RABBITMQ_URL=amqp://admin:password@rabbitmq:5672
AUTH_SERVICE_URL=http://auth-service:50051
```
