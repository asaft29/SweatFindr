# Notification Service

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#4dabf7', 'edgeLabelBackground': '#1f2937', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Clients["Clients"]
        FE["Frontend - WebSocket Client"]
    end

    subgraph NotificationService["Notification Service<br/>Port 8004"]
        WS["WebSocket Server - Axum"]
        CONN_MGR["Connection Manager - DashMap"]
        BROKER[Message Broker]
    end

    subgraph External["External Services"]
        AUTH["Auth Service - gRPC"]
        RMQ[RabbitMQ]
    end

    FE <-->|WebSocket| WS

    WS -->|validate token| AUTH
    WS --> CONN_MGR

    RMQ -->|ws.broadcast| BROKER
    BROKER --> CONN_MGR
    CONN_MGR -->|push to user| WS

    style WS fill:#ff922b,stroke:#e8590c,color:#fff
    style RMQ fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style AUTH fill:#4dabf7,stroke:#1971c2,color:#fff
```

Real-time notification delivery via WebSocket with RabbitMQ message consumption.

## How It Works

The Notification Service bridges RabbitMQ messages to connected WebSocket clients for real-time updates.

**WebSocket Connection:**
1. Frontend connects to `ws://host:8004/ws?token={jwt}`
2. Service validates token via Auth Service gRPC
3. On success, stores connection in DashMap (user_id → WebSocket sender)
4. Ping/pong keepalive every 30 seconds

**Message Broadcasting:**
1. Consumes `ws.broadcast` messages from RabbitMQ
2. Extracts target user ID from message
3. Looks up user's WebSocket connection
4. Pushes message to connected client

**Message Types:**
- `RefundStatusChanged` - Sent to client when refund is approved/rejected
- `NewRefundRequest` - Sent to event owner when client requests refund

## WebSocket Endpoint

```
GET ws://localhost:8004/ws?token={jwt_token}
```

## Message Format

**Refund Status Changed:**
```json
{
  "type": "refund_status_changed",
  "request_id": 123,
  "ticket_cod": "TKT-ABC123",
  "status": "APPROVED",
  "event_name": "Summer Festival",
  "message": null
}
```

**New Refund Request:**
```json
{
  "type": "new_refund_request",
  "request_id": 123,
  "ticket_cod": "TKT-ABC123",
  "requester_email": "user@example.com",
  "reason": "Cannot attend"
}
```

## RabbitMQ Integration

**Consumes:**
- Queue: `ws.broadcast.queue`
- Routing key: `ws.broadcast`
- Consumer tag: `notification_ws_broker`

**Message Routing:**
- `RefundStatusChanged` → broadcasts to `user_id` (client)
- `NewRefundRequest` → broadcasts to `event_owner_id`

## Connection Manager

DashMap for concurrent connection storage:
```rust
connections: Arc<DashMap<UserId, Vec<(ConnectionId, tokio::sync::mpsc::UnboundedSender<Message>)>>>
```
> [!NOTE]
> - **UnboundedSender:**  I had to ensure that sending messages to WebSocket clients is **non-blocking**. A slow WebSocket client should never block the message broker consumer or affect other clients.
> - **Vector of Connections:** A single user may have multiple active sessions (e.g., multiple tabs, mobile + desktop). Notifications must be synchronized across all their devices and/or tabs which are opened.

## Environment Variables

```bash
RABBITMQ_URL=amqp://admin:password@rabbitmq:5672
AUTH_SERVICE_URL=http://auth-service:50051
```
