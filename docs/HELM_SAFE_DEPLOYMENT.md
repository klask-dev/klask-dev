# Safe Deployment Guide for Klask Helm Chart

This guide provides step-by-step instructions for safely deploying and upgrading Klask using Helm, with safeguards against StatefulSet immutable field errors.

---

## Table of Contents

1. [Initial Deployment](#initial-deployment)
2. [Safe Upgrades](#safe-upgrades)
3. [Validation Before Deployment](#validation-before-deployment)
4. [Troubleshooting](#troubleshooting)
5. [Emergency Recovery](#emergency-recovery)

---

## Initial Deployment

### Prerequisites

Before deploying Klask, ensure you have:

- Kubernetes cluster running (1.18+)
- `kubectl` configured and access to target cluster
- `helm` 3.0+ installed
- PostgreSQL accessible (internal or external)

### Step 1: Create Namespace

```bash
kubectl create namespace klask --kubeconfig ~/.kube/test
```

### Step 2: Create Custom Values File

Create a copy of the default values:

```bash
cp charts/klask/values.yaml my-values.yaml
```

Or use the provided examples:

```bash
# For external database
cp charts/klask/values-external-db.yaml my-values.yaml

# For local development
cp charts/klask/values-local.yaml my-values.yaml
```

Edit `my-values.yaml` with your specific configuration:

```yaml
backend:
  image:
    tag: "0.2.0"
  resources:
    limits:
      memory: 1Gi

frontend:
  image:
    tag: "0.2.0"

ingress:
  enabled: true
  hosts:
    - host: klask.yourdomain.com
      paths:
        - path: /
          pathType: Prefix
```

### Step 3: Validate Configuration

Before deploying, validate that there are no conflicting values:

```bash
# Check for immutable field issues (fresh deployment should be fine)
./.claude/validators/helm-values-validator.sh my-values.yaml
```

### Step 4: Perform Dry-Run

Always test the deployment plan first:

```bash
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f my-values.yaml \
  --namespace klask \
  --install \
  --kubeconfig ~/.kube/test \
  --dry-run \
  --debug > /tmp/helm-plan.yaml

# Review the plan
cat /tmp/helm-plan.yaml
```

### Step 5: Deploy

Once you're confident, deploy for real:

```bash
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f my-values.yaml \
  --namespace klask \
  --install \
  --kubeconfig ~/.kube/test \
  --wait \
  --timeout 5m
```

### Step 6: Verify Deployment

```bash
# Check pod status
kubectl get pods -n klask --kubeconfig ~/.kube/test

# Expected output:
# klask-backend-xxxxx      1/1  Running
# klask-frontend-xxxxx     1/1  Running
# klask-postgresql-0       1/1  Running

# Check services
kubectl get svc -n klask --kubeconfig ~/.kube/test

# Check StatefulSet (PostgreSQL)
kubectl get statefulset -n klask --kubeconfig ~/.kube/test
```

### Step 7: Verify Application Health

```bash
# Backend health check
kubectl exec -it klask-backend-xxxxx -n klask --kubeconfig ~/.kube/test -- \
  curl -s http://localhost:3000/api/status

# Frontend health check
kubectl exec -it klask-frontend-xxxxx -n klask --kubeconfig ~/.kube/test -- \
  curl -s http://localhost:8080/health

# Database connection
kubectl exec -it klask-postgresql-0 -n klask --kubeconfig ~/.kube/test -- \
  psql -U klask -d klask -c "SELECT COUNT(*) FROM repositories;"
```

---

## Safe Upgrades

### Checklist Before Each Upgrade

Use this checklist before every upgrade:

- [ ] Review release notes and CHANGELOG
- [ ] Test upgrade in `test` environment first
- [ ] Run validation script: `./.claude/validators/helm-values-validator.sh`
- [ ] Check current cluster state: `helm status klask -n klask`
- [ ] Backup database (if using internal PostgreSQL): `pg_dump ...`
- [ ] Create backup of current configuration: `helm get values klask -n klask > backup-values.yaml`
- [ ] Review changes with `--dry-run --debug`

### Step 1: Validate Current Deployment State

```bash
# Check current release status
helm status klask -n klask --kubeconfig ~/.kube/test

# Save current values
helm get values klask -n klask --kubeconfig ~/.kube/test > current-values.yaml

# Check StatefulSet current configuration
kubectl get statefulset klask-postgresql -n klask \
  --kubeconfig ~/.kube/test \
  -o yaml | grep -E "serviceName|storageClass"
```

### Step 2: Prepare New Values

Create a new values file with only the changes:

```bash
# Copy current values
cp current-values.yaml new-values.yaml

# Edit with your new configuration
# Example: update image tags
vim new-values.yaml
```

**Critical:** Make sure you're NOT changing immutable fields:

```yaml
# ❌ DON'T CHANGE THESE:
nameOverride: "klask"           # Same as before
fullnameOverride: ""            # Same as before

postgresql:
  auth:
    username: "klask"           # Same as before
    database: "klask"           # Same as before
  persistence:
    storageClass: ""            # Same as before (if set)

# ✅ OK TO CHANGE:
backend:
  image:
    tag: "0.2.1"                # Safe
  resources:
    limits:
      memory: 2Gi               # Safe

frontend:
  image:
    tag: "0.2.1"                # Safe
```

### Step 3: Validate Changes

```bash
# Check for immutable field violations
./.claude/validators/helm-values-validator.sh \
  -n klask \
  new-values.yaml

# Output should be:
# ✓ No violations detected
# or show what fields are safe/unsafe
```

### Step 4: Dry-Run Upgrade

```bash
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.1 \
  -f new-values.yaml \
  --namespace klask \
  --kubeconfig ~/.kube/test \
  --dry-run \
  --debug > /tmp/upgrade-plan.yaml

# Review the plan
diff -u <(helm template klask -f current-values.yaml) \
        <(helm get manifest klask -n klask) | head -50
```

### Step 5: Execute Upgrade

If dry-run looks good:

```bash
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.1 \
  -f new-values.yaml \
  --namespace klask \
  --kubeconfig ~/.kube/test \
  --wait \
  --timeout 5m
```

### Step 6: Monitor Rollout

```bash
# Watch deployment progress
kubectl rollout status deployment/klask-backend -n klask --kubeconfig ~/.kube/test

# Watch pod creation
kubectl get pods -n klask --kubeconfig ~/.kube/test -w

# View logs if issues occur
kubectl logs -n klask -l app.kubernetes.io/name=klask-backend --kubeconfig ~/.kube/test -f
```

### Step 7: Verify Upgrade Success

```bash
# Check all pods are running
kubectl get pods -n klask --kubeconfig ~/.kube/test

# Verify no rollback occurred
helm status klask -n klask --kubeconfig ~/.kube/test
# Should show: STATUS: deployed

# Test application endpoints
kubectl port-forward -n klask svc/klask-backend 3000:3000 --kubeconfig ~/.kube/test &
curl -s http://localhost:3000/api/status | jq .

kubectl port-forward -n klask svc/klask-frontend 8080:8080 --kubeconfig ~/.kube/test &
curl -s http://localhost:8080/health
```

---

## Validation Before Deployment

### Using the Validation Script

The validation script checks for immutable field violations:

```bash
# Basic usage
./.claude/validators/helm-values-validator.sh my-values.yaml

# With custom namespace
./.claude/validators/helm-values-validator.sh -n production my-values.yaml

# With custom release name
./.claude/validators/helm-values-validator.sh -r my-klask my-values.yaml

# Full options
./.claude/validators/helm-values-validator.sh \
  -n klask \
  -r klask \
  -k ~/.kube/test \
  my-values.yaml
```

### Script Output

The script will output:

```
🔍 Helm StatefulSet Immutable Fields Validator

📋 Current StatefulSet configuration:
  serviceName: klask-postgresql-headless
  selector.matchLabels:
    {
      "app.kubernetes.io/name": "klask-postgresql"
    }
  persistence.storageClass: default
  persistence.size: 2Gi

📊 Fields that CANNOT be modified:
  ✗ serviceName
  ✗ selector.matchLabels
  ✗ volumeClaimTemplates
  ✗ podManagementPolicy

⚙️  Fields that CAN be modified safely:
  ✓ template (pod spec)
  ✓ replicas
  ✓ updateStrategy
  ✓ persistence.size (only increase)
  ✓ resources
  ✓ image tag

🔎 Analyzing your values file...
[Any warnings or errors]
```

---

## Troubleshooting

### Error: "Forbidden: updates to statefulset spec"

```
Error: UPGRADE FAILED: cannot patch "klask-postgresql" with kind StatefulSet:
StatefulSet.apps "klask-postgresql" is invalid: spec: Forbidden: updates to
statefulset spec for fields other than 'replicas', 'ordinals', 'template',
'updateStrategy', 'persistentVolumeClaimRetentionPolicy' and 'minReadySeconds'
are forbidden
```

**Cause:** You're trying to change an immutable field.

**Solution:**

1. **Identify which field changed:**
   ```bash
   helm diff upgrade klask ... -f new-values.yaml
   ```

2. **Revert the change in your values file:**
   ```bash
   # Restore the original value for the immutable field
   helm get values klask -n klask > current-values.yaml
   # Copy the immutable value from current-values.yaml
   ```

3. **Or, delete and recreate if you must change it:**
   ```bash
   # Backup
   kubectl get statefulset,pvc -n klask -o yaml > /tmp/backup.yaml

   # Delete (preserves PVC)
   kubectl delete statefulset klask-postgresql --cascade=orphan -n klask

   # Wait for cleanup
   sleep 5

   # Retry upgrade
   helm upgrade klask ... -f new-values.yaml
   ```

### Error: "Pod never reached ready state"

```
Timed out waiting for pod klask-backend-xxxxx to become ready
```

**Solution:**

1. Check pod status:
   ```bash
   kubectl describe pod klask-backend-xxxxx -n klask --kubeconfig ~/.kube/test
   ```

2. Check pod logs:
   ```bash
   kubectl logs klask-backend-xxxxx -n klask --kubeconfig ~/.kube/test
   ```

3. Common causes:
   - Database not accessible: Check `backend.env.database.url`
   - Image pull failure: Check registry credentials
   - Resource limits: Check if cluster has enough resources

### Error: "StatefulSet rollout timeout"

**Solution:**

```bash
# Check current StatefulSet status
kubectl get statefulset klask-postgresql -n klask --kubeconfig ~/.kube/test

# Watch pod creation
kubectl get pods -n klask --kubeconfig ~/.kube/test -w

# Increase timeout on next attempt
helm upgrade klask ... --timeout 10m
```

---

## Emergency Recovery

### If Something Goes Wrong

#### Option 1: Rollback to Previous Version

```bash
# View release history
helm history klask -n klask --kubeconfig ~/.kube/test

# Rollback to previous release
helm rollback klask 1 -n klask --kubeconfig ~/.kube/test --wait

# Verify rollback
helm status klask -n klask --kubeconfig ~/.kube/test
```

#### Option 2: Manual StatefulSet Deletion (Data Preserved)

If upgrade fails and pods are stuck:

```bash
# Delete StatefulSet but preserve PVC (data safe)
kubectl delete statefulset klask-postgresql \
  --cascade=orphan \
  -n klask \
  --kubeconfig ~/.kube/test

# Wait for cleanup
sleep 5

# Retry upgrade with original values
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f original-values.yaml \
  -n klask \
  --kubeconfig ~/.kube/test \
  --wait
```

#### Option 3: Full Uninstall and Reinstall

**WARNING:** This deletes everything including PVCs (data loss risk!)

```bash
# Backup first!
kubectl get all,pvc,statefulsets -n klask -o yaml > /tmp/backup-$(date +%s).yaml
pg_dump -h localhost -U klask klask > /tmp/klask-backup-$(date +%s).sql

# Uninstall
helm uninstall klask -n klask --kubeconfig ~/.kube/test

# Wait for cleanup
sleep 10

# Reinstall
helm install klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f my-values.yaml \
  -n klask \
  --kubeconfig ~/.kube/test \
  --wait
```

---

## Advanced: Automated Safe Deployment Script

Create a script to automate the safe deployment process:

```bash
#!/bin/bash
# File: scripts/safe-helm-deploy.sh

set -euo pipefail

NAMESPACE="${1:-klask}"
VALUES_FILE="${2:-my-values.yaml}"
CHART="oci://ghcr.io/klask-dev/klask:0.2.0"
KUBECONFIG="${KUBECONFIG:-$HOME/.kube/test}"
DRY_RUN="${DRY_RUN:-false}"

echo "🚀 Safe Helm Deployment for Klask"
echo "=================================="
echo "Namespace: $NAMESPACE"
echo "Values: $VALUES_FILE"
echo "Chart: $CHART"
echo ""

# 1. Validate
echo "1️⃣  Validating configuration..."
./.claude/validators/helm-values-validator.sh -n "$NAMESPACE" "$VALUES_FILE"
echo "✓ Validation passed"
echo ""

# 2. Backup current state
echo "2️⃣  Backing up current state..."
mkdir -p /tmp/klask-backups
BACKUP_ID=$(date +%s)
helm get values klask -n "$NAMESPACE" > "/tmp/klask-backups/values-$BACKUP_ID.yaml" 2>/dev/null || true
kubectl get statefulset,pvc,secret -n "$NAMESPACE" -o yaml > "/tmp/klask-backups/resources-$BACKUP_ID.yaml" 2>/dev/null || true
echo "✓ Backup saved to /tmp/klask-backups/resources-$BACKUP_ID.yaml"
echo ""

# 3. Dry-run
echo "3️⃣  Running dry-run..."
if helm upgrade klask "$CHART" \
    -f "$VALUES_FILE" \
    --namespace "$NAMESPACE" \
    --kubeconfig "$KUBECONFIG" \
    --dry-run \
    --debug > /tmp/helm-dry-run-$BACKUP_ID.yaml 2>&1; then
  echo "✓ Dry-run successful"
else
  echo "✗ Dry-run failed! Check /tmp/helm-dry-run-$BACKUP_ID.yaml"
  exit 1
fi
echo ""

# 4. Confirm
if [ "$DRY_RUN" = "true" ]; then
  echo "4️⃣  DRY-RUN mode enabled - stopping here"
  echo "Review plan: /tmp/helm-dry-run-$BACKUP_ID.yaml"
  exit 0
fi

read -p "4️⃣  Proceed with deployment? (yes/no) " -r
if [[ ! $REPLY =~ ^[Yy]es$ ]]; then
  echo "Aborted."
  exit 1
fi

# 5. Deploy
echo "5️⃣  Deploying..."
helm upgrade klask "$CHART" \
  -f "$VALUES_FILE" \
  --namespace "$NAMESPACE" \
  --kubeconfig "$KUBECONFIG" \
  --install \
  --wait \
  --timeout 5m

echo "✓ Deployment complete"
echo ""

# 6. Verify
echo "6️⃣  Verifying..."
kubectl get pods -n "$NAMESPACE" --kubeconfig "$KUBECONFIG"
echo ""
echo "✓ All done!"
echo "Backup ID: $BACKUP_ID"
echo "Restore with: helm get values klask -n $NAMESPACE > /tmp/klask-backups/values-$BACKUP_ID.yaml"

# Usage:
# ./scripts/safe-helm-deploy.sh klask my-values.yaml
# DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml
```

Make it executable:

```bash
chmod +x scripts/safe-helm-deploy.sh

# Use it
./scripts/safe-helm-deploy.sh klask my-values.yaml

# Or dry-run first
DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml
```

---

## Summary

| Task | Command |
|------|---------|
| **Validate** | `./.claude/validators/helm-values-validator.sh my-values.yaml` |
| **Dry-run** | `helm upgrade klask ... --dry-run --debug` |
| **Deploy** | `helm upgrade klask ... --install --wait` |
| **Verify** | `kubectl get pods -n klask` |
| **Rollback** | `helm rollback klask 1` |
| **Recover** | `kubectl delete statefulset klask-postgresql --cascade=orphan` |

---

## See Also

- [IMMUTABLE_FIELDS.md](./IMMUTABLE_FIELDS.md) - Detailed reference of immutable fields
- [Helm Documentation](https://helm.sh/docs/)
- [Kubernetes StatefulSet Docs](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/)
