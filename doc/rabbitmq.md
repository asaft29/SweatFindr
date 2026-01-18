# RabbitMQ

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#4dabf7', 'edgeLabelBackground': '#1f2937', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Publishers["Publishers"]
        CLIENT[Client Service]
        EVENT[Event Service]
    end

    subgraph RabbitMQ["RabbitMQ - Port 5672"]
        EX{refund.exchange<br/>Topic Exchange}
        Q1[refund.requested.queue]
        Q2[refund.resolved.email.queue]
        Q3[refund.resolved.client.queue]
        Q4[ws.broadcast.queue]
    end

    subgraph Consumers["Consumers"]
        EVENT2[Event Service]
        EMAIL[Email Service]
        CLIENT2[Client Service]
        NOTIF[Notification Service]
    end

    CLIENT -->|"refund.requested"| EX
    EVENT -->|"refund.resolved"| EX
    EVENT -->|"ws.broadcast"| EX

    EX -->|"refund.requested"| Q1
    EX -->|"refund.resolved"| Q2
    EX -->|"refund.resolved"| Q3
    EX -->|"ws.broadcast"| Q4

    Q1 --> EVENT2
    Q2 --> EMAIL
    Q3 --> CLIENT2
    Q4 --> NOTIF

    style EX fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style CLIENT fill:#ffd43b,stroke:#f59f00,color:#000
    style EVENT fill:#69db7c,stroke:#2f9e44,color:#fff
    style EMAIL fill:#da77f2,stroke:#9c36b5,color:#fff
    style NOTIF fill:#ff922b,stroke:#e8590c,color:#fff
```

Message broker for async communication between services using topic exchange pattern.

## How It Works

RabbitMQ enables decoupled, asynchronous communication for the refund workflow. Services publish messages to a topic exchange, which routes them to queues based on routing keys.

**Exchange:** `refund.exchange` (Topic type, durable)

**Pattern:** Publisher → Exchange → Queue(s) → Consumer(s)

---

## Message Flow

### 1. Refund Request Flow

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#4dabf7', 'signalColor': '#4dabf7', 'signalTextColor': '#4dabf7', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
sequenceDiagram
    participant U as User
    participant CS as Client Service
    participant RMQ as RabbitMQ
    participant ES as Event Service
    participant NS as Notification Service
    participant O as Event Owner

    U->>CS: Request refund
    CS->>RMQ: publish(refund.requested)
    RMQ->>ES: consume from refund.requested.queue
    ES->>ES: Create refund record (PENDING)
    ES->>RMQ: publish(ws.broadcast)
    RMQ->>NS: consume from ws.broadcast.queue
    NS->>O: WebSocket: New refund request
```

### 2. Refund Resolution Flow

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#4dabf7', 'signalColor': '#4dabf7', 'signalTextColor': '#4dabf7', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
sequenceDiagram
    participant O as Event Owner
    participant ES as Event Service
    participant RMQ as RabbitMQ
    participant EMS as Email Service
    participant CS as Client Service
    participant NS as Notification Service
    participant U as User

    O->>ES: Approve/Reject refund
    ES->>RMQ: publish(refund.resolved)
    ES->>RMQ: publish(ws.broadcast)

    RMQ->>EMS: consume from refund.resolved.email.queue
    EMS->>U: Send approval/rejection email
    RMQ->>CS: consume from refund.resolved.client.queue
    CS->>CS: Update ticket status
    RMQ->>NS: consume from ws.broadcast.queue
    NS->>U: WebSocket: Status changed
```

---

## Queues & Routing

| Queue | Routing Key | Consumer | Purpose |
|-------|-------------|----------|---------|
| `refund.requested.queue` | `refund.requested` | Event Service | Process new refund requests |
| `refund.resolved.email.queue` | `refund.resolved` | Email Service | Send approval/rejection emails |
| `refund.resolved.client.queue` | `refund.resolved` | Client Service | Update ticket status in MongoDB |
| `ws.broadcast.queue` | `ws.broadcast` | Notification Service | Push real-time WebSocket updates |

---

## Message Types

### RefundRequested
```json
{
  "request_id": 123,
  "ticket_cod": "TKT-ABC123",
  "requester_id": 1,
  "requester_email": "user@example.com",
  "event_id": 5,
  "event_owner_id": 2,
  "reason": "Cannot attend"
}
```

### RefundResolved
```json
{
  "request_id": 123,
  "ticket_cod": "TKT-ABC123",
  "requester_email": "user@example.com",
  "status": "APPROVED",
  "event_name": "Summer Festival",
  "message": null
}
```

### WebSocketMessage
```json
{
  "type": "refund_status_changed",
  "request_id": 123,
  "ticket_cod": "TKT-ABC123",
  "status": "APPROVED",
  "user_id": 1
}
```