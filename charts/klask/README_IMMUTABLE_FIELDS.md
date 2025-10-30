# Klask Helm Chart - Immutable Fields & Safe Deployment

This document explains the immutable fields protection in Kubernetes StatefulSets and how to safely deploy Klask without encountering the "Forbidden: updates to statefulset spec" error.

---

## Quick Start

### Before Every Deployment

1. **Validate your configuration:**
   ```bash
   ./.claude/validators/helm-values-validator.sh my-values.yaml
   ```

2. **Run a dry-run:**
   ```bash
   helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
     -f my-values.yaml \
     --namespace klask \
     --dry-run --debug
   ```

3. **Deploy safely:**
   ```bash
   ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
   ```

---

## What Are Immutable Fields?

Kubernetes StatefulSets have certain fields that **cannot be modified** after initial deployment. This is a protection mechanism to prevent data loss and cluster inconsistency.

### Common Error

```
Error: UPGRADE FAILED: cannot patch "klask-postgresql" with kind StatefulSet:
StatefulSet.apps "klask-postgresql" is invalid: spec: Forbidden: updates to
statefulset spec for fields other than 'replicas', 'ordinals', 'template',
'updateStrategy', 'persistentVolumeClaimRetentionPolicy' and 'minReadySeconds'
are forbidden
```

### What This Means

You tried to change a field that Kubernetes doesn't allow modifying after the StatefulSet was created.

---

## Immutable Fields in Klask

| Field | Immutable? | Notes |
|-------|-----------|-------|
| `nameOverride` | ❌ Yes | Affects serviceName |
| `fullnameOverride` | ❌ Yes | Affects serviceName |
| `postgresql.auth.username` | ❌ Yes | Immutable after init |
| `postgresql.auth.database` | ❌ Yes | Immutable after init |
| `postgresql.persistence.storageClass` | ❌ Yes | Defined in volumeClaimTemplates |
| `postgresql.persistence.size` | ✅ No | Can increase (never decrease) |
| `postgresql.image.tag` | ✅ No | Safe to change |
| `backend.image.tag` | ✅ No | Safe to change |
| `backend.resources` | ✅ No | Safe to change |
| `*.livenessProbe` | ✅ No | Safe to change |

---

## Documentation Files

### 📚 [IMMUTABLE_FIELDS.md](./IMMUTABLE_FIELDS.md)
**Detailed reference guide** covering:
- Complete list of immutable vs. mutable fields
- Why each field is immutable
- What happens if you try to change them
- How to recover if you accidentally changed one
- Best practices for PostgreSQL configuration
- Summary table with all fields

**When to read:** When you need to understand a specific field or troubleshoot an error

### 📖 [SAFE_DEPLOYMENT.md](./SAFE_DEPLOYMENT.md)
**Step-by-step deployment guide** covering:
- Initial deployment process
- Safe upgrade procedure
- Validation before deployment
- Troubleshooting common errors
- Emergency recovery procedures
- Automated deployment script examples

**When to read:** Before deploying or upgrading Klask

---

## Tools

### Validator Script

Located at: `./.claude/validators/helm-values-validator.sh`

This script automatically checks your values file for immutable field violations.

**Usage:**
```bash
# Basic validation
./.claude/validators/helm-values-validator.sh my-values.yaml

# With namespace
./.claude/validators/helm-values-validator.sh -n production my-values.yaml

# Full help
./.claude/validators/helm-values-validator.sh -h
```

**What it checks:**
- Existing StatefulSet configuration
- Proposed changes in values file
- Warns about immutable field changes
- Suggests recovery steps

### Safe Deployment Script

Located at: `./scripts/safe-helm-deploy.sh`

This script provides a complete guided deployment process.

**Usage:**
```bash
# Deploy with all checks
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0

# Dry-run first
DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0

# Without backups
NO_BACKUP=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

**Features:**
- ✅ Prerequisite validation (kubectl, helm, kubeconfig)
- ✅ Immutable fields validation
- ✅ Automatic backups
- ✅ Dry-run with detailed output
- ✅ Manual confirmation before deployment
- ✅ Post-deployment verification

---

## Common Scenarios

### Scenario 1: Initial Deployment

No existing StatefulSet, so all fields are safe to modify.

```bash
# Just validate and deploy
./.claude/validators/helm-values-validator.sh my-values.yaml
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f my-values.yaml \
  --namespace klask \
  --install \
  --wait
```

### Scenario 2: Update Image Tags

Image tags can be changed freely.

```bash
# Update backend and frontend versions
sed -i 's/tag: "0.2.0"/tag: "0.2.1"/g' my-values.yaml

# Validate (no issues expected)
./.claude/validators/helm-values-validator.sh my-values.yaml

# Deploy
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.1
```

### Scenario 3: Scale Storage

Storage size can be increased (never decreased).

```bash
# Increase PostgreSQL storage from 2Gi to 10Gi
sed -i 's/size: 2Gi/size: 10Gi/g' my-values.yaml

# Validate
./.claude/validators/helm-values-validator.sh my-values.yaml

# Deploy
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

### Scenario 4: Change Database Name (WRONG!)

❌ **DO NOT DO THIS** - it will cause an error

```bash
# ❌ NEVER change these:
# postgresql:
#   auth:
#     database: "my-new-database"  # ← IMMUTABLE!

# If you need a different database:
# 1. Delete StatefulSet: kubectl delete statefulset klask-postgresql --cascade=orphan
# 2. Then deploy with new database name
```

---

## Troubleshooting

### Q: "Forbidden: updates to statefulset spec" error

**A:** You're trying to change an immutable field. Check [IMMUTABLE_FIELDS.md](./IMMUTABLE_FIELDS.md) to see which field changed, then:

1. Revert the change in your values file
2. Or delete the StatefulSet and recreate it:
   ```bash
   kubectl delete statefulset klask-postgresql --cascade=orphan -n klask
   helm upgrade klask ... (with new values)
   ```

### Q: How do I change PostgreSQL username/database?

**A:** These fields are immutable. To change them:

1. Backup your data:
   ```bash
   pg_dump -U klask -h localhost klask > backup.sql
   ```

2. Delete and recreate:
   ```bash
   kubectl delete statefulset klask-postgresql --cascade=orphan -n klask
   sleep 5
   helm upgrade klask ... (with new credentials)
   ```

3. Restore data if needed

### Q: Can I decrease storage size?

**A:** No. Storage size can only increase. To shrink it, you would need to:

1. Backup data
2. Delete StatefulSet and PVC
3. Recreate with smaller size

**Not recommended** - just increase storage instead.

### Q: How do I know which field caused the error?

**A:** Use the validator script with detailed output:

```bash
./.claude/validators/helm-values-validator.sh -n klask my-values.yaml
```

It will show you:
- Current StatefulSet configuration
- What fields cannot be changed
- What fields can be changed safely

---

## Best Practices

### 1. Always Validate Before Deploying

```bash
./.claude/validators/helm-values-validator.sh my-values.yaml
```

### 2. Use Version Control for Values Files

```bash
git add my-values.yaml
git commit -m "chore: update Klask configuration for staging"
```

This helps track changes between deployments.

### 3. Use `existingSecret` for PostgreSQL

Separates configuration from secrets:

```yaml
postgresql:
  auth:
    existingSecret: "klask-postgresql-credentials"
    username: ""      # ← Leave empty
    password: ""      # ← Leave empty
    database: ""      # ← Leave empty
```

### 4. Test in Lower Environment First

```bash
# Test in development
./scripts/safe-helm-deploy.sh klask-dev my-values.yaml 0.2.1

# If successful, deploy to production
./scripts/safe-helm-deploy.sh klask-prod my-values.yaml 0.2.1
```

### 5. Keep Release Notes

Before upgrading, always check:

```bash
# View current release info
helm status klask -n klask

# Check what will change
helm diff upgrade klask oci://ghcr.io/klask-dev/klask:0.2.1 -f my-values.yaml
```

---

## References

- **Immutable Fields Details:** [IMMUTABLE_FIELDS.md](./IMMUTABLE_FIELDS.md)
- **Deployment Guide:** [SAFE_DEPLOYMENT.md](./SAFE_DEPLOYMENT.md)
- **Kubernetes StatefulSet Docs:** https://kubernetes.io/docs/concepts/workloads/controllers/statefulset/
- **Helm Best Practices:** https://helm.sh/docs/chart_best_practices/

---

## Getting Help

If you encounter issues:

1. **Check the documentation:**
   - Read [IMMUTABLE_FIELDS.md](./IMMUTABLE_FIELDS.md) for field reference
   - Read [SAFE_DEPLOYMENT.md](./SAFE_DEPLOYMENT.md) for troubleshooting

2. **Run the validator:**
   ```bash
   ./.claude/validators/helm-values-validator.sh my-values.yaml
   ```

3. **Check error messages:**
   ```bash
   helm status klask -n klask
   kubectl describe statefulset klask-postgresql -n klask
   kubectl logs klask-postgresql-0 -n klask
   ```

4. **Ask for help:**
   - Open an issue on GitHub
   - Check the project's discussion board

---

## Summary

| What | Where | When |
|------|-------|------|
| Learn about immutable fields | [IMMUTABLE_FIELDS.md](./IMMUTABLE_FIELDS.md) | Need to understand a field |
| Deploy Klask safely | [SAFE_DEPLOYMENT.md](./SAFE_DEPLOYMENT.md) | Ready to deploy |
| Validate before deploying | `./.claude/validators/helm-values-validator.sh` | Before every `helm upgrade` |
| Quick guided deployment | `./scripts/safe-helm-deploy.sh` | Want automation and checks |

**Start here:** Read [SAFE_DEPLOYMENT.md](./SAFE_DEPLOYMENT.md) for step-by-step instructions.

---

*Last updated: 2025-10-27*
