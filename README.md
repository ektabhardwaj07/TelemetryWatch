# TelemetryWatch

TelemetryWatch is an open-source observability platform for collecting, storing, and visualizing metrics and system telemetry across cloud-native, Kubernetes, and traditional infrastructure. Built on Prometheus, Grafana, and PostgreSQL, TelemetryWatch provides a unified monitoring experience.

**🚀 [Live Demo](#deployment)** - Deploy your own instance on Railway or Render

**🎯 Platform Control Plane** - Manage multiple Supabase OSS projects with a unified control plane

## Architecture

TelemetryWatch consists of four main components:

- **TelemetryWatch Application**: Rust-based service that collects and exposes metrics via Prometheus-compatible endpoints
- **Prometheus**: Time-series database for metrics collection and storage
- **Grafana**: Visualization and dashboard platform
- **PostgreSQL**: Metadata and configuration storage

### System Architecture

```mermaid
graph TB
    subgraph "TelemetryWatch Platform Control Plane"
        TW[TelemetryWatch App<br/>Rust Service]
        WEB[Web UI Dashboard<br/>http://localhost:8080]
        API[REST API<br/>/api/v1/platform/projects]
    end
    
    subgraph "Observability Stack"
        PROM[Prometheus<br/>Metrics Collection]
        GRAF[Grafana<br/>Dashboards & Visualization]
    end
    
    subgraph "TelemetryWatch Data Storage"
        PG[(PostgreSQL<br/>Project Metadata<br/>& Config)]
    end
    
    subgraph "Supabase OSS Projects (Tenants)"
        SUP1[Supabase Project 1<br/>Kong + Postgres + Auth<br/>+ Project DB]
        SUP2[Supabase Project 2<br/>Kong + Postgres + Auth<br/>+ Project DB]
        SUPN[Supabase Project N...<br/>Each with own Postgres]
    end
    
    WEB --> TW
    API --> TW
    TW -->|Stores project metadata| PG
    TW -->|Exposes metrics| PROM
    PROM -->|Queries metrics| GRAF
    TW -->|Manages lifecycle| SUP1
    TW -->|Manages lifecycle| SUP2
    TW -->|Manages lifecycle| SUPN
    
    style TW fill:#667eea,color:#fff
    style WEB fill:#764ba2,color:#fff
    style PROM fill:#e6522c,color:#fff
    style GRAF fill:#f46800,color:#fff
    style PG fill:#336791,color:#fff
```

### Platform Control Plane Flow

```mermaid
sequenceDiagram
    participant Operator
    participant WebUI as Web UI
    participant API as TelemetryWatch API
    participant DB as PostgreSQL
    participant Metrics as Prometheus
    participant Supabase as Supabase OSS
    
    Operator->>WebUI: Register new project
    WebUI->>API: POST /api/v1/platform/projects
    API->>DB: Store project metadata
    DB-->>API: Project created
    API-->>WebUI: Project registered
    
    Operator->>WebUI: Suspend project
    WebUI->>API: POST /api/v1/platform/projects/{id}/suspend
    API->>DB: Update status = 'suspended'
    DB-->>API: Status updated
    API->>Supabase: (Future: Enforce suspension)
    API-->>WebUI: Project suspended
    
    API->>Metrics: Expose platform_projects metrics
    Metrics->>Grafana: Query metrics
    Grafana->>Operator: Show platform overview
```

## Features

- **Metrics Collection**: Prometheus-compatible metrics endpoint with comprehensive observability
- **Health Monitoring**: Health and readiness endpoints for Kubernetes
- **Database Integration**: PostgreSQL for storing platform project metadata (can use Supabase PostgreSQL or standalone)
- **Supabase Support**: Can use Supabase's managed PostgreSQL as TelemetryWatch's database, while managing multiple Supabase OSS project instances
- **Platform Control Plane**: Multi-project management for Supabase OSS projects
  - Web UI dashboard for project management
  - REST API for programmatic access
  - Register and manage Supabase projects
  - Lifecycle management (active/suspended)
  - Per-project observability and metrics
- **Visualization**: Pre-configured Grafana dashboards with platform overview
- **Containerized**: Docker Compose for local development
- **Kubernetes Ready**: Complete K8s manifests for production deployment

## Prerequisites

- **For Local Development**:
  - Docker and Docker Compose
  - Rust 1.75+ (if building locally)
  - (Optional) Supabase account for managed database

- **For Kubernetes Deployment**:
  - Kubernetes cluster (1.20+)
  - kubectl configured
  - PersistentVolume support

## Quick Start

### Using Docker Compose

1. Clone the repository:
```bash
git clone <repository-url>
cd TelemetryWatch
```

2. **Choose your database option:**

   **Option A: Local PostgreSQL (Default)**
   ```bash
   docker-compose up -d
   ```

   **Option B: Supabase (Optional)**
   - Create a Supabase project at [supabase.com](https://supabase.com)
   - Get your connection string from Supabase dashboard (Settings → Database)
   - Create `.env` file and set `DATABASE_URL` to your Supabase connection string:
     ```bash
     cp env.example .env
     # Edit .env and set DATABASE_URL to your Supabase connection string
     ```
   - Start services (PostgreSQL will be skipped):
     ```bash
     docker-compose -f docker-compose.yml -f docker-compose.supabase.yml up -d
     ```

   **Note**: If using Option A, services are already started. For Option B, services start with the command above.

3. Create a `.env` file (optional, for custom passwords):
   ```bash
   cp env.example .env
   # Edit .env with your passwords
   ```

4. Access the services:
   - **TelemetryWatch Web UI**: http://localhost:8080 (Platform Control Plane Dashboard)
   - **TelemetryWatch API Docs**: http://localhost:8080/api/docs/ (Interactive Swagger UI)
   - **TelemetryWatch API**: http://localhost:8080/api/v1/...
   - **Prometheus**: http://localhost:9090
   - **Grafana**: http://localhost:3000 (default: admin/admin12345 - **CHANGE IN PRODUCTION!**)
   - **PostgreSQL**: localhost:5433 (TelemetryWatch), localhost:5432 (Supabase if running)

5. Check service health:
```bash
curl http://localhost:8080/health
curl http://localhost:8080/ready
curl http://localhost:8080/metrics
```

### Building from Source

1. Install Rust dependencies:
```bash
cargo build --release
```

2. Set up environment variables (create `.env` file):
```bash
HOST=0.0.0.0
PORT=8080
DATABASE_URL=postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch
DATABASE_MAX_CONNECTIONS=10
METRICS_ENABLED=true
```

3. Start PostgreSQL (if not using Docker):
```bash
docker run -d \
  --name postgresql \
  -e POSTGRES_USER=telemetrywatch \
  -e POSTGRES_PASSWORD=telemetrywatch \
  -e POSTGRES_DB=telemetrywatch \
  -p 5432:5432 \
  postgres:15-alpine
```

4. Run the application:
```bash
cargo run --release
```

## Kubernetes Deployment

1. Apply all manifests:
```bash
kubectl apply -f k8s/namespace.yaml
kubectl apply -f k8s/pvc.yaml
kubectl apply -f k8s/configmaps.yaml
kubectl apply -f k8s/postgresql-deployment.yaml
kubectl apply -f k8s/telemetrywatch-deployment.yaml
kubectl apply -f k8s/prometheus-deployment.yaml
kubectl apply -f k8s/grafana-deployment.yaml
kubectl apply -f k8s/services.yaml
```

2. Or apply all at once:
```bash
kubectl apply -f k8s/
```

3. Check deployment status:
```bash
kubectl get pods -n telemetrywatch
kubectl get services -n telemetrywatch
```

4. Access Grafana (NodePort service):
```bash
# Get the NodePort
kubectl get svc grafana-service -n telemetrywatch

# Access via <node-ip>:<nodeport>
```

## API Endpoints

### API Documentation
- **Swagger UI**: `GET /api/docs/` - Interactive API documentation (note: trailing slash required)
  - Local: http://localhost:8080/api/docs/
  - Railway: https://telemetrywatch-production-22dc.up.railway.app/api/docs/
- **OpenAPI Spec**: `GET /api/openapi.json` - OpenAPI 3.0 specification

### Core Endpoints
- `GET /health` - Health check endpoint
- `GET /ready` - Readiness check endpoint (includes database check)
- `GET /metrics` - Prometheus metrics endpoint
- `GET /api/v1/status` - Application status with database health

### Platform Control Plane API
- `GET /api/v1/platform/projects` - List all registered Supabase projects
- `POST /api/v1/platform/projects` - Register a new Supabase project
  ```json
  {
    "name": "Project Name",
    "slug": "project-slug",
    "plan": "dev|pro|enterprise",
    "region": "us-east-1",
    "db_url": "postgresql://...",
    "api_base_url": "http://..."
  }
  ```
- `POST /api/v1/platform/projects/{id}/suspend` - Suspend a project
- `POST /api/v1/platform/projects/{id}/resume` - Resume a suspended project

## Configuration

Configuration is managed through environment variables:

| Variable | Description | Default |
|----------|-------------|---------|
| `HOST` | Server bind address | `0.0.0.0` |
| `PORT` | Server port | `8080` |
| `DATABASE_URL` | PostgreSQL connection string | `postgresql://telemetrywatch:telemetrywatch@localhost:5432/telemetrywatch` |
| `DATABASE_MAX_CONNECTIONS` | Max database connections | `10` |
| `METRICS_ENABLED` | Enable metrics collection | `true` |

## Project Structure

```
TelemetryWatch/
├── src/
│   ├── main.rs          # Application entry point
│   ├── api.rs           # HTTP API routes (including platform control plane)
│   ├── config.rs        # Configuration management
│   ├── db.rs            # PostgreSQL integration and schema
│   ├── metrics.rs       # Prometheus metrics definitions
│   ├── middleware.rs    # HTTP middleware for metrics collection
│   └── platform.rs      # Platform control plane (Supabase project management)
├── config/
│   ├── prometheus.yml   # Prometheus configuration
│   └── grafana/         # Grafana provisioning and dashboards
├── docker/
│   └── Dockerfile       # Application container
├── k8s/                 # Kubernetes manifests
├── docker-compose.yml   # Local development setup
└── Cargo.toml          # Rust dependencies
```

## Development

### Running Tests

```bash
cargo test
```

### Building Docker Image

```bash
docker build -f docker/Dockerfile -t telemetrywatch:latest .
```

### Code Formatting

```bash
cargo fmt
```

### Linting

```bash
cargo clippy
```

## Metrics

TelemetryWatch exposes the following Prometheus metrics:

### HTTP Metrics
- `http_requests_total` - Total HTTP requests (labeled by method, endpoint, status)
- `http_request_duration_seconds` - HTTP request duration (labeled by method, endpoint)
- `http_errors_total` - HTTP errors (labeled by method, endpoint, status, error_type)
- `http_request_size_bytes` - Request payload size (labeled by method, endpoint)
- `http_response_size_bytes` - Response payload size (labeled by method, endpoint, status)
- `sla_violations_total` - SLA violations (labeled by endpoint, sla_type)

### Database Metrics
- `database_queries_total` - Total database queries
- `database_query_duration_seconds` - Database query duration
- `db_pool_size` - Database connection pool size
- `db_pool_idle` - Idle connections in pool
- `db_pool_active` - Active connections in pool

### Platform Metrics
- `platform_projects` - Platform project status (labeled by slug, status, plan, region)
- `platform_projects_total` - Total projects by status and plan

### System Metrics
- `active_connections` - Number of active HTTP connections

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Security

**⚠️ Important**: Default passwords are provided for development only. For production deployments, you **MUST** change all passwords and use proper secret management. See [SECURITY.md](SECURITY.md) for detailed security guidelines.

## License

This project is licensed under the MIT License.

## Deployment

### Railway Deployment

1. **Install Railway CLI** (optional):
   ```bash
   npm i -g @railway/cli
   ```

2. **Login to Railway**:
   ```bash
   railway login
   ```

3. **Initialize project**:
   ```bash
   railway init
   ```

4. **Add PostgreSQL service**:
   - Go to Railway dashboard
   - Click "New" → "Database" → "Add PostgreSQL"
   - Copy the `DATABASE_URL` from the service variables

5. **Set environment variables**:
   - In Railway dashboard, go to your service → **Variables** tab
   - Add `DATABASE_PUBLIC_URL` with the value from your PostgreSQL service's `DATABASE_PUBLIC_URL` variable
   - Or use `DATABASE_URL` with the internal URL (`telemetrywatch-db.railway.internal`)
   - Railway automatically provides `PORT` - no need to set it manually
   - Optional: Set `DATABASE_MAX_CONNECTIONS=10` and `METRICS_ENABLED=true`

6. **Deploy**:
   - Railway will automatically deploy when you push to GitHub
   - Or manually trigger deployment from the Railway dashboard
   - Your app will be available at `https://your-service-name.up.railway.app`

### Render Deployment

1. **Create a new Web Service** on [Render Dashboard](https://dashboard.render.com)

2. **Connect your GitHub repository**

3. **Configure the service**:
   - **Build Command**: (leave empty, uses Dockerfile)
   - **Start Command**: `/usr/local/bin/telemetrywatch`
   - **Environment**: `docker`

4. **Add PostgreSQL database**:
   - Create a new PostgreSQL database
   - Copy the `Internal Database URL`

5. **Set environment variables**:
   - `DATABASE_URL`: Your PostgreSQL connection string
   - `HOST`: `0.0.0.0`
   - `PORT`: `$PORT` (Render provides this)
   - `DATABASE_MAX_CONNECTIONS`: `10`
   - `METRICS_ENABLED`: `true`

6. **Deploy**: Render will automatically build and deploy from your `render.yaml`

### Environment Variables for Production

Make sure to set these in your hosting platform:

| Variable | Description | Required | Notes |
|----------|-------------|----------|-------|
| `DATABASE_URL` or `DATABASE_PUBLIC_URL` | PostgreSQL connection string | ✅ Yes | Railway provides both - use `DATABASE_PUBLIC_URL` if `DATABASE_URL` is empty |
| `HOST` | Server bind address | No | Default: `0.0.0.0` |
| `PORT` | Server port | ✅ Yes | Automatically provided by Railway/Render |
| `DATABASE_MAX_CONNECTIONS` | Max DB connections | No | Default: `10` |
| `METRICS_ENABLED` | Enable metrics | No | Default: `true` |

**Note**: For Railway, if `DATABASE_URL` variable reference doesn't work, manually set `DATABASE_PUBLIC_URL` with the connection string from your PostgreSQL service.

## Support

For issues and questions, please open an issue on GitHub.
