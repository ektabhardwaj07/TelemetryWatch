# TelemetryWatch

TelemetryWatch is an open-source observability platform for collecting, storing, and visualizing metrics and system telemetry across cloud-native, Kubernetes, and traditional infrastructure. Built on Prometheus, Grafana, and PostgreSQL, TelemetryWatch provides a unified monitoring experience.

## Architecture

TelemetryWatch consists of four main components:

- **TelemetryWatch Application**: Rust-based service that collects and exposes metrics via Prometheus-compatible endpoints
- **Prometheus**: Time-series database for metrics collection and storage
- **Grafana**: Visualization and dashboard platform
- **PostgreSQL**: Metadata and configuration storage

```
┌─────────────────┐
│  TelemetryWatch │
│   (Rust App)    │
└────────┬────────┘
         │
         ├─── Exposes /metrics ───> Prometheus
         │
         └─── Stores metadata ───> PostgreSQL
                                      │
                                      │
         Grafana <─── Queries ──── Prometheus
            │
            └─── Reads config ──── PostgreSQL
```

## Features

- **Metrics Collection**: Prometheus-compatible metrics endpoint
- **Health Monitoring**: Health and readiness endpoints for Kubernetes
- **Database Integration**: PostgreSQL for metadata storage (local or Supabase)
- **Supabase Support**: Optional integration with Supabase managed PostgreSQL
- **Visualization**: Pre-configured Grafana dashboards
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
   - **TelemetryWatch API**: http://localhost:8080
   - **Prometheus**: http://localhost:9090
   - **Grafana**: http://localhost:3000 (default: admin/admin12345 - **CHANGE IN PRODUCTION!**)
   - **PostgreSQL**: localhost:5432

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

- `GET /health` - Health check endpoint
- `GET /ready` - Readiness check endpoint (includes database check)
- `GET /metrics` - Prometheus metrics endpoint
- `GET /api/v1/status` - Application status with database health

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
│   ├── api.rs           # HTTP API routes
│   ├── config.rs        # Configuration management
│   ├── db.rs            # PostgreSQL integration
│   └── metrics.rs       # Prometheus metrics
├── config/
│   ├── prometheus.yml   # Prometheus configuration
│   └── grafana/         # Grafana provisioning
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

- `http_requests_total` - Total HTTP requests (labeled by method, endpoint, status)
- `http_request_duration_seconds` - HTTP request duration (labeled by method, endpoint)
- `active_connections` - Number of active connections
- `database_queries_total` - Total database queries
- `database_query_duration_seconds` - Database query duration

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

## Security

**⚠️ Important**: Default passwords are provided for development only. For production deployments, you **MUST** change all passwords and use proper secret management. See [SECURITY.md](SECURITY.md) for detailed security guidelines.

## License

[Add your license here]

## Support

For issues and questions, please open an issue on GitHub.
