# Client Service

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#9ca3af', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Clients["Clients"]
        FE[Frontend]
    end

    subgraph ClientService["Client Service - Port 8002"]
        REST[REST API<br/>Axum]
        HANDLERS[Handlers]
        REPOS[Repositories]
        CONSUMER[Refund Consumer]
        PUBLISHER[Message Publisher]
    end

    subgraph External["External Services"]
        AUTH[Auth Service<br/>gRPC]
        EMAIL[Email Service<br/>gRPC]
        EVENT[Event Service<br/>REST]
        RMQ[RabbitMQ]
    end

    subgraph Storage["Storage"]
        MONGO[(MongoDB<br/>Clients DB)]
    end

    subgraph Upstream["Upstream Publisher"]
        EVENT2[Event Service]
    end

    subgraph Downstream["Downstream Consumers"]
        EVENT3[Event Service]
    end

    FE --> REST

    REST --> HANDLERS
    HANDLERS --> AUTH
    HANDLERS --> EMAIL
    HANDLERS --> EVENT
    HANDLERS --> REPOS
    HANDLERS --> PUBLISHER
    REPOS --> MONGO

    RMQ -->|refund.resolved| CONSUMER
    CONSUMER --> REPOS
    PUBLISHER -->|refund.requested| RMQ

    EVENT2 -.->|refund.resolved| RMQ
    RMQ -.->|refund.requested| EVENT3

    style REST fill:#ffd43b,stroke:#f59f00,color:#000
    style RMQ fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style MONGO fill:#13aa52,stroke:#0e8a42,color:#fff
    style AUTH fill:#4dabf7,stroke:#1971c2,color:#fff
    style EMAIL fill:#da77f2,stroke:#9c36b5,color:#fff
    style EVENT fill:#69db7c,stroke:#2f9e44,color:#fff
    style EVENT2 fill:#69db7c,stroke:#2f9e44,color:#fff
    style EVENT3 fill:#69db7c,stroke:#2f9e44,color:#fff
```

Client profile management and ticket purchasing service with REST API.

## How It Works

The *client-service* manages user profiles and orchestrates ticket purchases across services.

**Profile Management:**
- Stores client profiles in MongoDB (name, email, ticket list)
- `/clients/me` endpoint returns current user's profile by extracting user ID from JWT and looking up email via Auth Service

**Ticket Purchase Flow:**
1. Frontend calls method for buying the ticket
2. Service generates unique ticket code
3. Calls *event-service* to reserve ticket
4. Adds ticket to client's `lista_bilete` array in MongoDB
5. Returns ticket details to frontend

**Refund Request Flow:**
1. Client requests refund via associated method
2. Service publishes `refund.requested` message to RabbitMQ
3. Consumes `refund.resolved` messages when *event-service* processes refund
4. On APPROVED: removes ticket from client's list
5. On REJECTED: marks ticket as rejected

## RabbitMQ Integration

**Publishes:**
- Routing key: `refund.requested`
- Message: `RefundRequested { ticket_cod, requester_id, event_id, reason, ... }`

**Consumes:**
- Queue: `refund.resolved.client.queue`
- Routing key: `refund.resolved`
- Actions: Remove ticket on APPROVED, mark REJECTED otherwise

## MongoDB Schema

```typescript
clients {
  _id: ObjectId,
  email: String (unique),
  prenume: String,
  nume: String,
  lista_bilete: [{ cod: String }]
}
```

## Environment Variables

```bash
MONGODB_URI=mongodb://user:pass@client-db:27017/clientsdb
RABBITMQ_URL=amqp://admin:password@rabbitmq:5672
AUTH_SERVICE_URL=http://auth-service:50051
EMAIL_SERVICE_URL=http://email-service:50052
EVENT_SERVICE_URL=http://event-service:8001
```
