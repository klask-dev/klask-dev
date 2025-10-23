# Klask Deployment Guide

This guide explains how to deploy Klask using Docker for production environments.

## Prerequisites

- Docker Engine 20.10+ 
- Docker Compose 2.0+
- Git
- At least 2GB RAM and 10GB disk space

## Quick Start

1. **Clone the repository**
   ```bash
   git clone https://github.com/klask-dev/klask-rs.git
   cd klask-rs
   ```

2. **Set up environment variables**
   ```bash
   cp .env.example .env
   # Edit .env and change ENCRYPTION_KEY to a secure random string
   nano .env
   ```

3. **Deploy with Docker Compose**
   ```bash
   # Deploy the full stack
   docker-compose up -d
   
   # Or deploy only essential services (without pgAdmin and Redis)
   docker-compose up -d postgres klask-backend klask-frontend
   ```

4. **Access the application**
   - Frontend: http://localhost
   - Backend API: http://localhost:3000
   - pgAdmin (optional): http://localhost:8080

## Environment Configuration

### Required Environment Variables

Edit the `.env` file before deployment:

```bash
# CRITICAL: Change this encryption key for production!
ENCRYPTION_KEY=your-super-secret-encryption-key-32-chars-minimum

# Database credentials (change for production)
POSTGRES_USER=klask
POSTGRES_PASSWORD=your-secure-password
POSTGRES_DB=klask_rs

# Application settings
RUST_LOG=info
PORT=3000
HOST=0.0.0.0
```

### Optional Services

The Docker Compose setup includes optional development and administration tools:

```bash
# Enable optional services (pgAdmin, Redis)
docker-compose --profile tools up -d

# Or specify individual services
docker-compose up -d postgres klask-backend klask-frontend pgadmin
```

## Production Deployment

### Security Considerations

1. **Change default credentials**
   - Update `ENCRYPTION_KEY` to a secure random string (32+ characters)
   - Change PostgreSQL credentials (`POSTGRES_USER`, `POSTGRES_PASSWORD`)
   - Update pgAdmin credentials if using the tools profile

2. **Network security**
   - Use a reverse proxy (nginx, Traefik) with SSL/TLS
   - Restrict database access to application containers only
   - Consider using Docker secrets for sensitive data

3. **Data persistence**
   - Database data is persisted in Docker volumes
   - Search index and application data are persisted in volumes
   - Back up these volumes regularly

### SSL/TLS Configuration

For production, use a reverse proxy to handle SSL termination:

```yaml
# docker-compose.override.yml example
version: '3.8'
services:
  klask-frontend:
    labels:
      - "traefik.enable=true"
      - "traefik.http.routers.klask.rule=Host(`your-domain.com`)"
      - "traefik.http.routers.klask.tls.certresolver=letsencrypt"
```

### Resource Limits

For production, consider adding resource limits:

```yaml
# docker-compose.override.yml example
version: '3.8'
services:
  klask-backend:
    deploy:
      resources:
        limits:
          memory: 2G
          cpus: '1.0'
        reservations:
          memory: 512M
          cpus: '0.5'
```

## Service Architecture

### Services Overview

- **postgres**: PostgreSQL database for application data
- **klask-backend**: Rust backend API server
- **klask-frontend**: React frontend served by nginx
- **pgadmin** (optional): Database administration interface
- **redis** (optional): Caching layer for future use

### Port Mapping

- Frontend: `80:8080` (nginx serving React app)
- Backend: `3000:3000` (Rust API server)
- Database: `5432:5432` (PostgreSQL)
- pgAdmin: `8080:80` (Database admin interface)
- Redis: `6379:6379` (Cache server)

### Data Volumes

- `postgres_data`: Database files
- `klask_data`: Application data and temporary files
- `klask_index`: Search index files
- `redis_data`: Redis persistence (if using Redis)

## Administration

### Health Checks

All services include health checks:

```bash
# Check service status
docker-compose ps

# View service logs
docker-compose logs klask-backend
docker-compose logs klask-frontend
```

### Database Management

Access the database directly:

```bash
# Connect to PostgreSQL
docker-compose exec postgres psql -U klask -d klask_rs

# Or use pgAdmin web interface
# Navigate to http://localhost:8080
# Login with admin@klask.dev / admin
```

### Backup and Restore

```bash
# Backup database
docker-compose exec postgres pg_dump -U klask klask_rs > backup.sql

# Backup volumes
docker run --rm -v klask-rs_postgres_data:/data -v $(pwd):/backup alpine tar czf /backup/postgres_backup.tar.gz /data

# Restore database
docker-compose exec -T postgres psql -U klask klask_rs < backup.sql
```

### Scaling

Scale individual services:

```bash
# Scale backend instances
docker-compose up -d --scale klask-backend=3

# Note: Frontend and database should remain as single instances
# Load balancing requires additional configuration
```

## Troubleshooting

### Common Issues

1. **Database connection errors**
   - Check if PostgreSQL container is healthy: `docker-compose ps`
   - Verify DATABASE_URL in backend environment
   - Check network connectivity between containers

2. **Frontend API errors**
   - Verify backend is accessible at `http://klask-backend:3000`
   - Check nginx configuration in frontend container
   - Review backend logs: `docker-compose logs klask-backend`

3. **Build failures**
   - Ensure Docker has sufficient resources (memory, disk)
   - Check for dependency conflicts in package.json or Cargo.toml
   - Clear Docker build cache: `docker builder prune`

### Logs and Debugging

```bash
# View all logs
docker-compose logs

# Follow logs for specific service
docker-compose logs -f klask-backend

# Debug container issues
docker-compose exec klask-backend sh
```

### Performance Monitoring

Monitor resource usage:

```bash
# Container resource usage
docker stats

# Detailed container inspection
docker-compose exec klask-backend top
```

## Development vs Production

### Development Mode

For development, use the existing development setup:

```bash
# Use development compose file
docker-compose -f docker-compose.yml up -d

# Run services locally
npm run dev  # Frontend
cargo run    # Backend
```

### Production Optimizations

The production Docker setup includes:

- Multi-stage builds for optimized image sizes
- Non-root user execution for security
- Health checks for service monitoring
- Proper signal handling for graceful shutdowns
- Resource-optimized nginx configuration
- Security headers and gzip compression

## Migration from Development

To migrate from a development setup:

1. Export existing data from development database
2. Update environment variables for production
3. Deploy using production Docker Compose
4. Import data into new production database
5. Update any hardcoded URLs or configurations

## Support

For deployment issues:
- Check the GitHub issues for known problems
- Review logs for specific error messages
- Ensure all prerequisites are met
- Verify environment variable configuration