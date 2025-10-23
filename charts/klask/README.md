# Klask Helm Chart

This Helm chart deploys the Klask code search engine on a Kubernetes cluster.

## Components

- **Frontend**: React-based web interface
- **Backend**: Rust-based API server with search capabilities
- **PostgreSQL**: Database for storing metadata and user data

## Prerequisites

- Kubernetes 1.16+
- Helm 3.0+
- Docker images for frontend and backend components

## Installation

### 1. Build Docker Images

First, build the Docker images for your application:

```bash
# Build backend image
cd klask-rs
docker build -t klask-backend:latest .

# Build frontend image
cd klask-react
docker build -t klask-frontend:latest .
```

### 2. Install the Chart

```bash
# Install the chart
helm install klask ./helm/klask
```

### 3. Custom Installation

You can override default values by creating a custom values file:

```bash
# Create custom-values.yaml
cat > custom-values.yaml << EOF
ingress:
  enabled: true
  hosts:
    - host: klask.yourdomail.com
      paths:
        - path: /
          pathType: Prefix

backend:
  image:
    repository: your-registry/klask-backend
    tag: v1.0.0

frontend:
  image:
    repository: your-registry/klask-frontend
    tag: v1.0.0
EOF

# Install with custom values
helm install klask ./helm/klask -f custom-values.yaml
```

## Configuration

The following table lists the configurable parameters and their default values:

### Global Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `replicaCount` | Number of replicas | `1` |

### Backend Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `backend.enabled` | Enable backend deployment | `true` |
| `backend.image.repository` | Backend image repository | `klask-backend` |
| `backend.image.tag` | Backend image tag | `latest` |
| `backend.service.port` | Backend service port | `3000` |

### Frontend Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `frontend.enabled` | Enable frontend deployment | `true` |
| `frontend.image.repository` | Frontend image repository | `klask-frontend` |
| `frontend.image.tag` | Frontend image tag | `latest` |
| `frontend.service.port` | Frontend service port | `8080` |

### PostgreSQL Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `postgresql.enabled` | Enable PostgreSQL deployment | `true` |
| `postgresql.auth.database` | Database name | `klask` |
| `postgresql.auth.username` | Database username | `klask` |
| `postgresql.auth.password` | Database password | `klask` |

### Ingress Parameters

| Parameter | Description | Default |
|-----------|-------------|---------|
| `ingress.enabled` | Enable ingress | `false` |
| `ingress.hosts[0].host` | Hostname | `klask.local` |

## Usage

After installation, you can access the application:

1. **Using port-forward** (for ClusterIP service):
   ```bash
   kubectl port-forward service/klask-frontend 8080:8080
   ```
   Then visit http://localhost:8080

2. **Using ingress** (if enabled):
   Visit the hostname configured in your ingress

3. **Check pod status**:
   ```bash
   kubectl get pods -l "app.kubernetes.io/instance=klask"
   ```

## Upgrading

```bash
helm upgrade klask ./helm/klask
```

## Uninstalling

```bash
helm uninstall klask
```

## Troubleshooting

### Common Issues

1. **Backend connection issues**: Check if PostgreSQL is running and accessible
   ```bash
   kubectl logs -l "app.kubernetes.io/component=backend"
   ```

2. **Frontend not loading**: Verify the backend service is accessible
   ```bash
   kubectl get svc
   kubectl logs -l "app.kubernetes.io/component=frontend"
   ```

3. **Database connection**: Check PostgreSQL logs
   ```bash
   kubectl logs -l "app.kubernetes.io/name=postgresql"
   ```

### Health Checks

The chart includes health checks for all components:
- Backend: `GET /api/status`
- Frontend: `GET /health`
- PostgreSQL: Built-in health checks

## Development

For development purposes, you can:

1. **Enable debug logging**:
   ```yaml
   backend:
     env:
       - name: RUST_LOG
         value: "debug"
   ```

2. **Use development images**:
   ```yaml
   backend:
     image:
       tag: "dev"
   frontend:
     image:
       tag: "dev"
   ```
