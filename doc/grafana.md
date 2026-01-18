# Grafana

```mermaid
%%{init: {'theme': 'dark', 'themeVariables': { 'primaryColor': '#1f2937', 'primaryTextColor': '#fff', 'primaryBorderColor': '#4b5563', 'lineColor': '#4dabf7', 'edgeLabelBackground': '#1f2937', 'secondaryColor': '#374151', 'tertiaryColor': '#1f2937', 'background': '#000000'}}}%%
flowchart TB
    subgraph Services["Backend Services"]
        GW[Gateway]
        EVENT[Event Service]
        CLIENT[Client Service]
        AUTH[Auth Service]
        NOTIF[Notification Service]
        EMAIL[Email Service]
    end

    subgraph Collectors["Collectors"]
        PROM["Prometheus<br/>Collects Metrics"]
        PROMTAIL["Promtail<br/>Collects Logs"]
    end

    subgraph Platform["Storage & Visualization"]
        LOKI[Loki]
        GRAFANA[Grafana<br/>Port 3000]
    end

    GW -.-> PROM
    EVENT -.-> PROM
    CLIENT -.-> PROM
    AUTH -.-> PROM
    NOTIF -.-> PROM
    EMAIL -.-> PROM

    GW -.-> PROMTAIL
    EVENT -.-> PROMTAIL
    CLIENT -.-> PROMTAIL
    AUTH -.-> PROMTAIL
    NOTIF -.-> PROMTAIL
    EMAIL -.-> PROMTAIL

    PROMTAIL --> LOKI
    PROM --> GRAFANA
    LOKI --> GRAFANA

    style GRAFANA fill:#f46800,stroke:#cc5500,color:#fff
    style PROM fill:#e6522c,stroke:#b8421f,color:#fff
    style LOKI fill:#f46800,stroke:#cc5500,color:#fff
    style PROMTAIL fill:#868e96,stroke:#495057,color:#fff

    linkStyle 0,1,2,3,4,5,13 stroke:#e6522c,stroke-width:2px
    linkStyle 6,7,8,9,10,11,12,14 stroke:#4dabf7,stroke-width:2px
```

Monitoring and visualization dashboard for service metrics and logs.

## How It Works

Grafana connects to Prometheus for metrics and Loki for logs.

### Metrics
*Prometheus* scrapes metrics from all backend services via the `/metrics` endpoint that each one exposes
**Collected:** Request count, latency, error rates, active connections.

### Logs
*Promtail* collects logs from Docker containers and pushes them to *Loki*. Grafana queries *Loki* to display logs alongside metrics.
**Collected:** Application logs, warnings, errors, panic traces.

---

## Dashboard Overview

<div align="center">

![Dashboard Overview 1](img/grafana/dashboard-overview-1.png)

</div>

<div align="center">

![Dashboard Overview 2](img/grafana/dashboard-overview-2.png)

</div>

<div align="center">

![Dashboard Overview 3](img/grafana/dashboard-overview-3.png)

</div>



---

## Access

```
URL: http://localhost:3000
Default credentials: admin / admin
```

## Configuration

Grafana configuration is located in `yamls/grafana/` directory with pre-configured dashboards and Prometheus data source.
