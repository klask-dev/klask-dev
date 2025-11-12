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

## Secrets Configuration

The chart manages sensitive data using Kubernetes Secret objects, with **separate secrets for infrastructure and application concerns**:

1. **Infrastructure Secrets** (Database):
   - PostgreSQL credentials (`klask-postgresql`)
   - External database URLs (`klask-backend`)

2. **Application Secrets** (Authentication):
   - Encryption key (`klask-backend-auth`)
   - JWT secret (`klask-backend-auth`)

This separation allows for:
- Independent rotation policies
- Different access control (DBAs vs. DevOps)
- Clearer security boundaries
- Integration with different secret managers per concern

### Application Authentication Secrets

The backend requires two critical secrets for authentication and encryption, managed in a **separate** Kubernetes Secret:

- **`ENCRYPTION_KEY`**: Used for encrypting sensitive data at rest (e.g., OAuth tokens)
- **`JWT_SECRET`**: Used for signing and validating JWT authentication tokens

**Secret name**: `klask-backend-auth`

**Auto-generation behavior**: If `ENCRYPTION_KEY` or `JWT_SECRET` are not provided, they will be automatically generated with random 32-character values on first deployment. These auto-generated values are **preserved across Helm upgrades** via `lookup()` function.

#### Method 1: Auto-generated (Default - Recommended for Most Cases)

Simply deploy without providing secrets - they will be auto-generated:

```bash
# Just install the chart, secrets will be auto-generated
helm install klask ./helm/klask
```

The secrets are generated with random 32-character values and preserved across upgrades. To regenerate them:

```bash
# Delete the auth secret to regenerate on next deployment
kubectl delete secret klask-backend-auth -n <namespace>
helm upgrade klask ./helm/klask
```

#### Method 2: Custom Generated Keys (Development/Testing)

Provide your own randomly generated keys via values file:

```bash
# Generate random 32-byte hex strings
ENCRYPTION_KEY=$(openssl rand -hex 32)
JWT_SECRET=$(openssl rand -hex 32)

# Create/update your custom-values.yaml
cat > custom-values.yaml << EOF
backend:
  auth:
    encryptionKey: "$ENCRYPTION_KEY"
    jwtSecret: "$JWT_SECRET"
EOF

# Install the chart with custom values
helm install klask ./helm/klask -f custom-values.yaml
```

Or directly via CLI:

```bash
helm install klask ./helm/klask \
  --set backend.auth.encryptionKey="$(openssl rand -hex 32)" \
  --set backend.auth.jwtSecret="$(openssl rand -hex 32)"
```

#### Method 3: External Secret Management (Recommended for Production with HSM/Vault)

Use existing Kubernetes Secrets created by external systems (HashiCorp Vault, Sealed Secrets, etc.):

```bash
# Create the secret manually or via your secret management system
kubectl create secret generic klask-backend-auth \
  --from-literal=ENCRYPTION_KEY="your-key-here" \
  --from-literal=JWT_SECRET="your-secret-here"

# Configure the chart to use the existing secret
cat > custom-values.yaml << EOF
backend:
  existingAuthSecret: "klask-backend-auth"
EOF

helm install klask ./helm/klask -f custom-values.yaml
```

### PostgreSQL Passwords

Similarly, PostgreSQL passwords can be managed via:

```yaml
postgresql:
  auth:
    # Auto-generated if empty (recommended for production)
    postgresPassword: ""
    password: ""

    # OR use existing secret
    existingSecret: "postgres-credentials"
```

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
