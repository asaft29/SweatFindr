# Email Service

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#9ca3af', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Clients["Clients (gRPC)"]
        GW[Gateway]
        CS[Client Service]
    end

    subgraph EmailService["Email Service - Port 50052"]
        GRPC[gRPC Server]
        HANDLERS[Handlers]
        CODE_MGR[Code Manager]
        MAILER[SMTP Sender<br/>Lettre]
        CONSUMER[Refund Consumer]
    end

    subgraph External["External"]
        RMQ[RabbitMQ]
        SMTP[Gmail SMTP]
    end

    subgraph Storage["Storage"]
        REDIS[(Redis<br/>Verification Codes)]
    end

    GW -->|SendVerificationEmail<br/>VerifyCode| GRPC
    CS -->|SendVerificationEmail<br/>ResendCode| GRPC

    GRPC --> HANDLERS
    HANDLERS --> CODE_MGR
    HANDLERS --> MAILER
    CODE_MGR --> REDIS
    MAILER --> SMTP

    RMQ -->|refund.resolved| CONSUMER
    CONSUMER --> MAILER

    style GRPC fill:#da77f2,stroke:#9c36b5,color:#fff
    style RMQ fill:#ff6b6b,stroke:#c92a2a,color:#fff
    style REDIS fill:#dc382d,stroke:#a52a2a,color:#fff
    style SMTP fill:#ea4335,stroke:#c5221f,color:#fff
```

Email delivery and verification code management via gRPC with async refund notifications.

## How It Works

The Email Service handles all outbound emails using Gmail SMTP (Lettre library). Verification codes are stored in Redis with TTL for automatic expiration.

---

## Email Types

### 1. Registration Verification Email

When a user registers, a 6-digit verification code is generated, stored in Redis (TTL - 15 minutes), and sent via email.

**Flow:** User registers → Client Service calls `SendVerificationEmail` → Code stored in Redis → Email sent → User enters code → `VerifyCode` validates against Redis

<div align="center">

![Verification Email](img/email-service/verification-email.png)

</div>

---

### 2. Resend Verification Email

If the code expires or user didn't receive it, they can request a new code. Old code is invalidated and new one generated.

<div align="center">

![Resend Verification](img/email-service/resend-verification.png)

</div>

---

### 3. Password Reset Email

User requests password reset → 6-digit code generated → Stored in Redis (15-minute TTL) → Email sent with reset code.

**Flow:** Forgot password → `SendPasswordResetEmail` → Code in Redis → User enters code → `VerifyPasswordResetCode` → Password updated

<div align="center">

![Password Reset Email](img/email-service/password-reset-email.png)

</div>

---

### 4. Refund Approved Email

When an event owner approves a refund request, the Email Service consumes the `refund.resolved` message from RabbitMQ and sends a confirmation email to the requester.

**Content:** Ticket code, event name, confirmation that refund was approved.

<div align="center">

![Refund Approved](img/email-service/refund-approved-email.png)

</div>

---

### 5. Refund Rejected Email

When a refund is rejected, the requester receives an email with the rejection reason provided by the event owner.

**Content:** Ticket code, event name, rejection message.

<div align="center">

![Refund Rejected](img/email-service/refund-rejected-email.png)

</div>

---

## gRPC Endpoints

| Method | Description |
|--------|-------------|
| `SendVerificationEmail` | Send verification code to email |
| `VerifyCode` | Validate verification code |
| `ResendVerificationCode` | Generate and send new code |
| `SendPasswordResetEmail` | Send password reset code |
| `VerifyPasswordResetCode` | Validate reset code |

## RabbitMQ Integration

**Consumes:**
- Queue: `refund.resolved.email.queue`
- Routing key: `refund.resolved`

## Environment Variables

```bash
REDIS_URL=redis://shared-redis:6379
RABBITMQ_URL=amqp://admin:password@rabbitmq:5672
SMTP_HOST=smtp.gmail.com
SMTP_PORT=587
SMTP_USERNAME=your-email@gmail.com
SMTP_PASSWORD=your-app-password
```
