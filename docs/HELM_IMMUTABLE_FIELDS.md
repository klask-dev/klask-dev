# Immutable Fields in Klask Helm Chart

## Overview

Kubernetes StatefulSets have immutable fields that **cannot be modified** after the initial deployment. Attempting to change these fields during a Helm upgrade will result in this error:

```
Error: UPGRADE FAILED: cannot patch "klask-postgresql" with kind StatefulSet:
StatefulSet.apps "klask-postgresql" is invalid: spec: Forbidden: updates to statefulset spec
for fields other than 'replicas', 'ordinals', 'template', 'updateStrategy',
'persistentVolumeClaimRetentionPolicy' and 'minReadySeconds' are forbidden
```

This is a **Kubernetes protection mechanism** to prevent accidental data loss and cluster inconsistency.

---

## Immutable Fields in Klask

### ❌ NEVER Change These Values Between Upgrades:

#### 1. **nameOverride** and **fullnameOverride**
```yaml
# In values.yaml or custom values
nameOverride: "klask"          # ❌ DO NOT CHANGE
fullnameOverride: "my-klask"   # ❌ DO NOT CHANGE
```

**Why:** These affect the StatefulSet's name and `spec.serviceName`, which are immutable.

**Impact:**
- Changes the headless service name (immutable)
- Breaks pod DNS discovery
- Causes upgrade failure

**Fix if needed:**
```bash
kubectl delete statefulset klask-postgresql --cascade=orphan -n <namespace>
# Then deploy with new values
helm upgrade klask ... -f new-values.yaml
```

---

#### 2. **postgresql.persistence.storageClass**
```yaml
postgresql:
  persistence:
    storageClass: "fast-ssd"   # ❌ DO NOT CHANGE from "default"
```

**Why:** Storage classes are defined in the `volumeClaimTemplates`, which are immutable.

**Impact:**
- Cannot switch between storage classes (e.g., "default" → "fast-ssd")
- PVCs will continue using the original storage class
- Upgrade will fail if values differ

**What you CAN do:**
- Leave it empty (uses cluster default)
- Keep it the same across upgrades
- Only change if creating a new cluster

**Fix if needed:**
```bash
# Backup data first
kubectl exec -it klask-postgresql-0 -n <namespace> -- pg_dump -U klask klask > backup.sql

# Delete StatefulSet (preserves PVC)
kubectl delete statefulset klask-postgresql --cascade=orphan -n <namespace>

# Deploy with new storage class
helm upgrade klask ... -f new-values.yaml --kubeconfig ~/.kube/test

# Restore data if needed
kubectl exec -it klask-postgresql-0 -n <namespace> -- psql -U klask klask < backup.sql
```

---

#### 3. **postgresql.auth.username** and **postgresql.auth.database**
```yaml
postgresql:
  auth:
    username: "klask"     # ❌ DO NOT CHANGE
    database: "klask"     # ❌ DO NOT CHANGE
```

**Why:** These are passed to the container at initialization and cannot be modified.

**Impact:**
- Application cannot connect if credentials change
- Database schema is tied to these values
- Upgrade will fail

**Best practice:** Use `existingSecret` instead:

```yaml
postgresql:
  auth:
    existingSecret: "klask-postgresql-credentials"
    # Leave these empty:
    username: ""
    password: ""
    database: ""
```

---

### ✅ Fields You CAN Change Safely:

#### 1. **postgresql.persistence.size**
```yaml
postgresql:
  persistence:
    size: 5Gi   # ✅ CAN INCREASE
```

- **Can increase:** `2Gi` → `5Gi` → `10Gi`
- **Cannot decrease:** `5Gi` → `2Gi` will fail
- PostgreSQL will use the expanded storage automatically

---

#### 2. **Image Tags**
```yaml
postgresql:
  image:
    tag: "18-alpine"  # ✅ CAN CHANGE (with caution)

backend:
  image:
    tag: "0.2.0"      # ✅ CAN CHANGE

frontend:
  image:
    tag: "0.2.0"      # ✅ CAN CHANGE
```

- Updates pods during rolling deployment
- Minor version changes are safer than major versions
- Always test in dev/test environment first

---

#### 3. **Resources (CPU/Memory)**
```yaml
postgresql:
  resources:
    limits:
      memory: 1Gi     # ✅ CAN CHANGE
    requests:
      memory: 512Mi   # ✅ CAN CHANGE

backend:
  resources:
    limits:
      memory: 2Gi     # ✅ CAN CHANGE
```

- Pods will be re-scheduled with new resource limits
- May cause eviction if cluster is full

---

#### 4. **Replicas**
```yaml
backend:
  replicaCount: 3     # ✅ CAN CHANGE
```

- Only for Deployments (backend/frontend)
- StatefulSet replicas are immutable without cascading delete

---

#### 5. **Probes, Affinity, Tolerations**
```yaml
backend:
  livenessProbe:
    initialDelaySeconds: 30   # ✅ CAN CHANGE
  readinessProbe:
    periodSeconds: 5          # ✅ CAN CHANGE

affinity: {}                  # ✅ CAN CHANGE
tolerations: []               # ✅ CAN CHANGE
```

---

## Safe Upgrade Checklist

Before running `helm upgrade`:

- [ ] Running validation script: `./.claude/validators/helm-values-validator.sh`
- [ ] Comparing values with current deployment
- [ ] Checking if any immutable fields are changing
- [ ] Database backup completed (if modifying postgres)
- [ ] Tested in `test` environment first
- [ ] All pods healthy before upgrading production

---

## Validation Script

Use the provided validation script to check for immutable field violations:

```bash
# Check before upgrade
./.claude/validators/helm-values-validator.sh ~/temp/custom-values.yml

# With custom namespace
./.claude/validators/helm-values-validator.sh -n my-namespace values.yaml

# With custom release name
./.claude/validators/helm-values-validator.sh -r my-release values.yaml
```

---

## Recovery: When You Need to Change Immutable Fields

If you absolutely must change an immutable field:

### Option 1: Delete StatefulSet (Preserve Data) - Recommended
```bash
# This deletes the StatefulSet but preserves the PVC
kubectl delete statefulset klask-postgresql --cascade=orphan -n <namespace> --kubeconfig ~/.kube/test

# Wait for completion
sleep 5

# Now upgrade with new values
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f ~/custom-values.yml \
  --namespace <namespace> \
  --kubeconfig ~/.kube/test \
  --wait
```

**Data safety:** ✅ Data preserved (PVC remains)
**Downtime:** 2-3 minutes

### Option 2: Full Delete (Data Loss - Backup First!)
```bash
# DANGEROUS: This deletes everything including data!
helm uninstall klask -n <namespace> --kubeconfig ~/.kube/test

# Restore from backup if needed
helm install klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f ~/custom-values.yml \
  --namespace <namespace> \
  --kubeconfig ~/.kube/test
```

**Data safety:** ❌ PVC deleted (data loss risk)
**Downtime:** 5+ minutes

---

## Best Practices

### 1. Use `existingSecret` for PostgreSQL
```yaml
postgresql:
  auth:
    existingSecret: "klask-postgresql-credentials"
    username: ""
    password: ""
    postgresPassword: ""
    database: ""
```

This separates configuration from secrets and makes upgrades safer.

### 2. Document Your Configuration
Create a `values-production.yaml` and commit it to version control (without secrets):

```yaml
# values-production.yaml
postgresql:
  persistence:
    size: 10Gi
    storageClass: "ebs-gp3"  # Defined once, never changed

backend:
  replicaCount: 3
  resources:
    limits:
      memory: 2Gi
```

### 3. Always Validate Before Upgrading
```bash
./.claude/validators/helm-values-validator.sh -n production values-production.yaml
helm upgrade klask ... --dry-run --debug
kubectl diff -f <(helm template klask ...)
```

### 4. Test in Test Environment First
```bash
./.claude/validators/helm-values-validator.sh -n klask-test values-test.yaml
helm upgrade klask ... -n klask-test --kubeconfig ~/.kube/test
# Verify everything works
# Only then proceed to production
```

---

## Summary Table

| Field | Mutable | Notes |
|-------|---------|-------|
| `nameOverride` | ❌ No | Affects serviceName |
| `fullnameOverride` | ❌ No | Affects serviceName |
| `postgresql.persistence.storageClass` | ❌ No | Defined in volumeClaimTemplates |
| `postgresql.persistence.size` | ✅ Yes (increase only) | Can expand PVC |
| `postgresql.auth.username` | ❌ No | Immutable after init |
| `postgresql.auth.database` | ❌ No | Immutable after init |
| `postgresql.image.tag` | ✅ Yes | Rolling deployment |
| `backend.image.tag` | ✅ Yes | Rolling deployment |
| `postgresql.resources` | ✅ Yes | Pod rescheduling |
| `backend.resources` | ✅ Yes | Pod rescheduling |
| `backend.replicaCount` | ✅ Yes | Rolling deployment |
| `*.livenessProbe` | ✅ Yes | Pod restart policy |
| `affinity` | ✅ Yes | Pod scheduling |

---

## References

- [Kubernetes StatefulSet Documentation](https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/)
- [Helm Best Practices](https://helm.sh/docs/chart_best_practices/)
- [PostgreSQL Helm Chart](https://github.com/bitnami/charts/tree/main/bitnami/postgresql)
