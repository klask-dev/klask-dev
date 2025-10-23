---
description: DevOps operations - deploy, monitor, troubleshoot Klask infrastructure
---

# 🚀 DevOps Command

This command provides comprehensive DevOps operations for Klask, including deployment, monitoring, troubleshooting, and infrastructure management.

## 🎯 What This Command Does

The `/devops` command handles all infrastructure and deployment operations:

1. **Deploy** - Deploy Klask to Kubernetes (test or production)
2. **Status** - Check health and status of all components
3. **Logs** - View and analyze application logs
4. **Troubleshoot** - Debug deployment and runtime issues
5. **Build** - Build and push Docker images
6. **CI/CD** - Check and manage GitHub Actions pipelines
7. **Database** - Manage PostgreSQL in Kubernetes
8. **Rollback** - Revert to previous deployment

---

## 📖 Usage Examples

### Deploy to Test Environment
```
/devops deploy test
```

### Check Application Status
```
/devops status
```

### View Backend Logs
```
/devops logs backend
```

### Troubleshoot Failing Pod
```
/devops troubleshoot pod klask-backend-xxx
```

### Build and Push Docker Images
```
/devops build all
```

### Check CI/CD Pipeline
```
/devops ci status
```

### Rollback Deployment
```
/devops rollback
```

---

## 🔧 Operations

### 1. Deploy 🚀

Deploy Klask to Kubernetes environment.

**Syntax:** `/devops deploy [environment]`

**Environments:**
- `test` - Test environment (uses `~/.kube/test`)
- `prod` - Production environment (not yet configured)

**Steps:**
1. Use `deployment-expert` agent
2. Build Docker images for backend and frontend
3. Push images to registry (if configured)
4. Deploy using Helm charts in `charts/klask/`
5. Wait for all pods to be ready
6. Run health checks
7. Display deployment status and endpoints

**Example:**
```
/devops deploy test
```

**Expected Output:**
```
🚀 Deploying Klask to test environment...

📦 Building Docker images...
✅ klask-backend:latest built successfully
✅ klask-frontend:latest built successfully

🎯 Deploying with Helm...
✅ Helm release 'klask' deployed

🔍 Checking pod status...
✅ klask-backend-xxxxx: Running (1/1)
✅ klask-frontend-xxxxx: Running (1/1)
✅ klask-postgres-xxxxx: Running (1/1)

🏥 Running health checks...
✅ Backend: http://localhost:8080/health - OK
✅ Frontend: http://localhost:3000 - OK
✅ Database: Connected

🎉 Deployment successful!
```

---

### 2. Status 📊

Check the health and status of all Klask components.

**Syntax:** `/devops status [component]`

**Components (optional):**
- `all` (default) - All components
- `backend` - Backend service only
- `frontend` - Frontend service only
- `database` - PostgreSQL database only
- `pods` - All pods status
- `services` - All services

**Steps:**
1. Use `deployment-expert` agent
2. Query Kubernetes cluster for pod status
3. Check service endpoints
4. Verify health endpoints
5. Display resource usage (CPU/Memory)
6. Show recent events

**Example:**
```
/devops status
```

**Expected Output:**
```
📊 Klask Status (test environment)

🟢 Backend (klask-backend)
  Pod: klask-backend-7d9f8b5c6-xxxxx
  Status: Running (1/1)
  Restarts: 0
  CPU: 45m / 500m
  Memory: 234Mi / 512Mi
  Health: ✅ OK

🟢 Frontend (klask-frontend)
  Pod: klask-frontend-6b8c7d5f4-xxxxx
  Status: Running (1/1)
  Restarts: 0
  CPU: 12m / 200m
  Memory: 89Mi / 256Mi
  Health: ✅ OK

🟢 Database (klask-postgres)
  Pod: klask-postgres-0
  Status: Running (1/1)
  Restarts: 0
  CPU: 23m / 1000m
  Memory: 456Mi / 1Gi
  Connections: 3/100

📡 Services:
  - klask-backend: ClusterIP (3000)
  - klask-frontend: NodePort (30080)
  - klask-postgres: ClusterIP (5432)

🎯 Overall Health: ✅ All systems operational
```

---

### 3. Logs 📜

View and analyze application logs.

**Syntax:** `/devops logs [component] [options]`

**Components:**
- `backend` - Backend application logs
- `frontend` - Frontend/Nginx logs
- `database` - PostgreSQL logs
- `all` - All component logs

**Options:**
- `--follow` or `-f` - Stream logs in real-time
- `--tail N` - Show last N lines (default: 100)
- `--since TIME` - Show logs since TIME (e.g., 5m, 1h)
- `--errors` - Filter for errors only

**Steps:**
1. Use `deployment-expert` agent
2. Identify the target pod(s)
3. Retrieve logs using `kubectl logs`
4. Filter and format logs if needed
5. Display with syntax highlighting

**Example:**
```
/devops logs backend --tail 50 --errors
```

**Expected Output:**
```
📜 Backend Logs (last 50 lines, errors only)

[2025-10-24 00:15:23] ERROR Failed to connect to database: connection timeout
[2025-10-24 00:15:45] ERROR Repository crawl failed: authentication error for repo 'test/project'
[2025-10-24 00:16:12] ERROR Search query failed: invalid syntax in query

📊 Summary:
  - Total errors: 3
  - Database errors: 1
  - Crawler errors: 1
  - Search errors: 1
```

---

### 4. Troubleshoot 🔍

Debug deployment and runtime issues.

**Syntax:** `/devops troubleshoot [issue-type]`

**Issue Types:**
- `pod [pod-name]` - Debug specific pod issues
- `deployment` - Debug deployment failures
- `database` - Debug database connection issues
- `performance` - Analyze performance issues
- `crashes` - Investigate crash loops
- `network` - Debug network/connectivity issues

**Steps:**
1. Use `deployment-expert` agent
2. Gather diagnostic information:
   - Pod status and events
   - Recent logs
   - Resource usage
   - Network connectivity
3. Analyze the issue
4. Provide diagnosis and recommendations
5. Suggest fixes or workarounds

**Example:**
```
/devops troubleshoot pod klask-backend-7d9f8b5c6-xxxxx
```

**Expected Output:**
```
🔍 Troubleshooting pod: klask-backend-7d9f8b5c6-xxxxx

📋 Pod Information:
  Status: CrashLoopBackOff
  Restarts: 5
  Age: 10m

⚠️ Recent Events:
  - Back-off restarting failed container (5m ago)
  - Container crashed with exit code 1 (6m ago)

📜 Last 20 Log Lines:
  [ERROR] Failed to connect to PostgreSQL: connection refused
  [INFO] Attempting to connect to klask-postgres:5432
  [ERROR] Database connection timeout after 30s
  [FATAL] Exiting due to database connection failure

🔧 Diagnosis:
  Issue: Backend cannot connect to PostgreSQL database
  Root Cause: Database service not ready or wrong hostname

💡 Recommended Fixes:
  1. Check if postgres pod is running:
     kubectl get pod klask-postgres-0 --kubeconfig ~/.kube/test

  2. Verify database service exists:
     kubectl get svc klask-postgres --kubeconfig ~/.kube/test

  3. Check connection string in backend config:
     Should be: postgresql://klask:klask@klask-postgres:5432/klask

  4. Restart backend after database is ready:
     kubectl rollout restart deployment/klask-backend --kubeconfig ~/.kube/test
```

---

### 5. Build 🏗️

Build and optionally push Docker images.

**Syntax:** `/devops build [component] [--push]`

**Components:**
- `backend` - Build backend Docker image
- `frontend` - Build frontend Docker image
- `all` - Build all images

**Options:**
- `--push` - Push images to registry after building
- `--tag TAG` - Use custom tag (default: latest)
- `--no-cache` - Build without using cache

**Steps:**
1. Use `deployment-expert` agent
2. Verify Dockerfiles exist
3. Build images with appropriate tags
4. Optionally push to registry
5. Display build summary with image sizes

**Example:**
```
/devops build all --push --tag v2.2.0
```

**Expected Output:**
```
🏗️ Building Docker images...

📦 Backend (klask-rs/Dockerfile)
  Building klask-backend:v2.2.0...
  ✅ Build successful (521MB)
  🚀 Pushing to registry...
  ✅ Pushed successfully

📦 Frontend (klask-react/Dockerfile)
  Building klask-frontend:v2.2.0...
  ✅ Build successful (89MB)
  🚀 Pushing to registry...
  ✅ Pushed successfully

📊 Summary:
  - Backend: klask-backend:v2.2.0 (521MB)
  - Frontend: klask-frontend:v2.2.0 (89MB)
  - Total size: 610MB
```

---

### 6. CI/CD 🔄

Check and manage GitHub Actions pipelines.

**Syntax:** `/devops ci [action]`

**Actions:**
- `status` - Check latest pipeline status
- `watch` - Watch current pipeline run
- `logs` - View pipeline logs
- `rerun` - Rerun failed workflow
- `cancel` - Cancel running workflow

**Steps:**
1. Use GitHub CLI (`gh`) to query workflows
2. Display workflow status and results
3. Provide links to GitHub Actions page
4. Optionally perform actions (rerun, cancel)

**Example:**
```
/devops ci status
```

**Expected Output:**
```
🔄 GitHub Actions Status

📋 Latest Workflow Runs:

CI (ci.yml)
  Run #245: ✅ Success (3m 42s ago)
  Branch: master
  Commit: f4fadd6 - "docs: enhance CLAUDE.md"
  Jobs:
    ✅ Backend Tests (Rust) - 2m 15s
    ✅ Frontend Tests (React) - 1m 48s
    ✅ Linting - 0m 32s

Build & Publish (build-and-publish.yml)
  Run #102: ✅ Success (1h ago)
  Branch: master
  Tag: v2.2.0
  Jobs:
    ✅ Build Backend - 5m 23s
    ✅ Build Frontend - 3m 12s
    ✅ Publish Images - 1m 45s

🔗 View on GitHub: https://github.com/klask-dev/klask/actions
```

---

### 7. Database 🗄️

Manage PostgreSQL database in Kubernetes.

**Syntax:** `/devops database [action]`

**Actions:**
- `status` - Check database status
- `connect` - Get connection command
- `backup` - Create database backup
- `restore [file]` - Restore from backup
- `logs` - View database logs
- `restart` - Restart database pod

**Steps:**
1. Use `deployment-expert` agent
2. Interact with postgres pod
3. Execute database operations
4. Display results

**Example:**
```
/devops database status
```

**Expected Output:**
```
🗄️ PostgreSQL Status

Pod: klask-postgres-0
Status: Running (1/1)
Age: 2d
Restarts: 0

📊 Database Info:
  Version: PostgreSQL 15.3
  Size: 2.4 GB
  Connections: 3/100
  Uptime: 2 days

📋 Tables:
  - repositories: 15 rows
  - users: 3 rows
  - (Tantivy index stored separately)

💡 To connect:
  kubectl exec -it klask-postgres-0 --kubeconfig ~/.kube/test -- psql -U klask -d klask

🔗 Connection String (from within cluster):
  postgresql://klask:klask@klask-postgres:5432/klask
```

---

### 8. Rollback ↩️

Revert to previous deployment version.

**Syntax:** `/devops rollback [revision]`

**Parameters:**
- `revision` (optional) - Specific revision number (default: previous)

**Steps:**
1. Use `deployment-expert` agent
2. Check deployment history
3. Confirm rollback revision
4. Execute Helm rollback
5. Wait for rollout to complete
6. Verify health

**Example:**
```
/devops rollback
```

**Expected Output:**
```
↩️ Rolling back Klask deployment...

📜 Deployment History:
  REVISION  DEPLOYED                STATUS      DESCRIPTION
  3         2025-10-24 00:30:15     deployed    Upgrade complete
  2         2025-10-23 18:45:22     superseded  Upgrade complete
  1         2025-10-23 12:00:00     superseded  Install complete

🎯 Rolling back to revision 2...

⏳ Waiting for rollout to complete...
  ✅ klask-backend: 1/1 pods updated
  ✅ klask-frontend: 1/1 pods updated

🏥 Running health checks...
  ✅ Backend: OK
  ✅ Frontend: OK

✅ Rollback successful! Currently at revision 2.
```

---

## ⚙️ Implementation

When this command is invoked:

1. **Parse the operation** from user request
2. **Launch deployment-expert agent** with specific task
3. **Execute operation** using kubectl, helm, docker, or gh CLI
4. **Monitor progress** and display real-time updates
5. **Verify success** with health checks
6. **Report results** with clear status and next steps

### Required Tools

The command assumes these tools are installed:
- `kubectl` - Kubernetes CLI
- `helm` - Helm package manager (optional, for deploy)
- `docker` - Docker CLI (for build operations)
- `gh` - GitHub CLI (for CI/CD operations)

### Kubernetes Configuration

For test environment operations, always use:
```bash
--kubeconfig ~/.kube/test
```

---

## 🎯 Common Use Cases

### 1. Deploy New Version
```
/devops build all --tag v2.3.0
/devops deploy test
/devops status
```

### 2. Debug Production Issue
```
/devops status
/devops logs backend --errors --tail 100
/devops troubleshoot performance
```

### 3. Database Backup Before Migration
```
/devops database backup
/devops database status
```

### 4. Rollback After Bad Deploy
```
/devops status  # Check current state
/devops rollback  # Revert to previous version
/devops logs backend --since 5m  # Verify recovery
```

### 5. Check CI/CD Before Release
```
/devops ci status
/devops build all --push --tag v2.3.0
# Wait for CI to pass, then deploy
```

---

## 🔒 Security Considerations

- **Secrets**: Never display or log sensitive credentials
- **RBAC**: Ensure proper Kubernetes permissions
- **Images**: Scan Docker images for vulnerabilities
- **Backups**: Encrypt database backups
- **Access**: Limit who can execute deploy/rollback commands

---

## 📊 Monitoring Integration

The deployment-expert agent should integrate with:
- **Kubernetes Events**: Monitor pod lifecycle events
- **Resource Metrics**: Track CPU/Memory usage
- **Application Logs**: Aggregate logs from all components
- **Health Endpoints**: Regular health check polling

---

## 💡 Tips

- Always check `/devops status` before deploying
- Use `/devops logs --errors` to quickly spot issues
- Create database backups before major changes
- Monitor CI/CD pipeline before deploying
- Test in `test` environment before production
- Keep deployment history for easy rollbacks

---

*This command streamlines all DevOps operations for Klask, from building to deploying to troubleshooting.*
