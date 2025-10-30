# Klask Deployment Guide

This guide covers deploying Klask to various environments.

## 📦 Table of Contents

- [Local Development](#local-development)
- [Docker Compose](#docker-compose)
- [Kubernetes (Helm)](#kubernetes-helm)
- [Production Best Practices](#production-best-practices)

---

## 🏠 Local Development

### Quick Start

```bash
# 1. Start PostgreSQL
docker-compose -f docker-compose.dev.yml up -d

# 2. Start backend
cd klask-rs
cargo run --bin klask-rs

# 3. Start frontend (in another terminal)
cd klask-react
npm run dev
```

Access:
- **Frontend**: http://localhost:5173
- **Backend API**: http://localhost:3000

---

## 🐳 Docker Compose

### Development Environment

```bash
# Start all services
docker-compose up -d

# View logs
docker-compose logs -f

# Stop services
docker-compose down

# With optional tools (pgAdmin, Redis)
docker-compose --profile tools up -d
```

### Production Environment

```bash
# Use production compose file
docker-compose -f docker-compose.prod.yml up -d
```

### Configuration

Create a `.env` file for environment variables:

```env
# Database
POSTGRES_USER=klask
POSTGRES_PASSWORD=your-secure-password
POSTGRES_DB=klask

# Backend
JWT_SECRET=your-jwt-secret-change-in-production
ENCRYPTION_KEY=your-encryption-key-change-in-production
RUST_LOG=info

# Frontend
VITE_API_URL=http://localhost:3000
```

---

## ☸️ Kubernetes (Helm)

### Prerequisites

- Kubernetes cluster (1.23+)
- Helm 3.x
- kubectl configured
- Storage class for persistent volumes
- Ingress controller (nginx, traefik, etc.)

### Quick Install

```bash
# Install with default values
helm install klask ./charts/klask

# Install with custom values
helm install klask ./charts/klask -f helm-example.yaml
```

### Basic Configuration

See [`helm-example.yaml`](helm-example.yaml) for a complete example. Here's a minimal setup:

```yaml
# minimal-values.yaml
backend:
  replicaCount: 1
  resources:
    requests:
      memory: "256Mi"
      cpu: "250m"

frontend:
  replicaCount: 1

postgresql:
  enabled: true
  auth:
    password: changeme

ingress:
  enabled: true
  className: nginx
  hosts:
    - host: klask.example.com
      paths:
        - path: /
          pathType: Prefix
  tls:
    - secretName: klask-tls
      hosts:
        - klask.example.com
```

Install:

```bash
helm install klask ./charts/klask -f minimal-values.yaml
```

### Helm Commands

```bash
# Install
helm install klask ./charts/klask -f values.yaml

# Upgrade
helm upgrade klask ./charts/klask -f values.yaml

# Rollback
helm rollback klask

# Uninstall
helm uninstall klask

# View status
helm status klask

# Validation before upgrade
helm upgrade klask ./charts/klask -f values.yaml --dry-run --debug
```

See [charts/klask/SAFE_DEPLOYMENT.md](charts/klask/SAFE_DEPLOYMENT.md) for detailed deployment safety guide.

---

## 🔒 Production Best Practices

### Security

1. **Change default passwords**
2. **Use Kubernetes secrets for sensitive data**
3. **Enable TLS/SSL with cert-manager**
4. **Configure network policies**

### Resource Management

1. **Set appropriate resource limits**
2. **Enable horizontal pod autoscaling**
3. **Use appropriate storage classes**

### Monitoring

1. **Enable Prometheus metrics**
2. **Configure health checks**
3. **Set up alerting**

### High Availability

1. **Run multiple replicas**
2. **Configure pod disruption budgets**
3. **Use anti-affinity rules**

See full guide: [charts/klask/README.md](charts/klask/README.md)

---

## 📚 Additional Resources

- [Helm Chart Documentation](charts/klask/README.md)
- [Helm Safety Guide](charts/klask/SAFE_DEPLOYMENT.md)
- [Immutable Fields Reference](charts/klask/IMMUTABLE_FIELDS.md)
- [Development Guide](CLAUDE.md)
