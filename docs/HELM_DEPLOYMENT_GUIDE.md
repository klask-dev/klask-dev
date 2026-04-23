# Klask Helm Deployment Guide

Quick reference guide for deploying Klask safely using Helm.

---

## 🚀 Before You Deploy

### 1. Read the Safety Guide (5 minutes)

Start with the quick reference to understand immutable fields:

```bash
cat charts/klask/README_IMMUTABLE_FIELDS.md
```

### 2. Validate Your Configuration (30 seconds)

Check that your values file doesn't violate immutable field constraints:

```bash
./.claude/validators/helm-values-validator.sh my-values.yaml
```

### 3. Prepare Kubernetes Namespace with Pod Security Standards

Before deploying, ensure your namespace enforces Pod Security Standards:

```bash
# Create namespace with PSS (Pod Security Standards) enforcement
kubectl create namespace klask --kubeconfig ~/.kube/test || true

# Apply Pod Security Standards labels (restricted policy)
kubectl label namespace klask \
  pod-security.kubernetes.io/enforce=restricted \
  pod-security.kubernetes.io/enforce-version=latest \
  pod-security.kubernetes.io/audit=restricted \
  pod-security.kubernetes.io/warn=restricted \
  --kubeconfig ~/.kube/test --overwrite

# Verify the labels are applied
kubectl get ns klask -o yaml --kubeconfig ~/.kube/test | grep pod-security
```

The `restricted` policy ensures:
- Containers run as non-root
- Read-only root filesystem is enforced
- Privileged escalation is prevented
- Linux capabilities are restricted

### 4. Deploy Safely

Use the automated deployment script:

```bash
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

---

## 📋 Common Tasks

### Initial Deployment

```bash
# 1. Create your values file
cp charts/klask/values.yaml my-values.yaml
vim my-values.yaml

# 2. Validate
./.claude/validators/helm-values-validator.sh my-values.yaml

# 3. Deploy
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

### Upgrade to New Version

```bash
# 1. Update version in values file (or use the script directly)
sed -i 's/tag: "0.2.0"/tag: "0.2.1"/' my-values.yaml

# 2. Validate changes
./.claude/validators/helm-values-validator.sh -n klask my-values.yaml

# 3. Test with dry-run first
DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.1

# 4. Deploy
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.1
```

### Increase Storage

```bash
# PostgreSQL storage is the only immutable field that can increase
# Editing values.yaml
sed -i 's/size: 2Gi/size: 10Gi/' my-values.yaml

# Validate (storage increase is allowed)
./.claude/validators/helm-values-validator.sh my-values.yaml

# Deploy
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

### Rollback to Previous Version

```bash
# If something goes wrong, rollback is simple:
helm rollback klask -n klask --kubeconfig ~/.kube/test

# Verify the rollback
helm status klask -n klask --kubeconfig ~/.kube/test
```

---

## ❌ What NOT to Do

### Never Change These After Initial Deployment

```yaml
# ❌ DO NOT CHANGE
nameOverride: "klask"
fullnameOverride: "my-klask"

postgresql:
  auth:
    username: "klask"              # Never change
    database: "klask"              # Never change
  persistence:
    storageClass: "my-storage"     # Never change
```

If you absolutely must change them:

```bash
# Delete StatefulSet (preserves data)
kubectl delete statefulset klask-postgresql --cascade=orphan -n klask --kubeconfig ~/.kube/test

# Then upgrade with new values
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

---

## 📚 Documentation

### For Different Situations

| Situation | Document | Read Time |
|-----------|----------|-----------|
| Need quick answers | `charts/klask/README_IMMUTABLE_FIELDS.md` | 5 min |
| Getting started | `charts/klask/SAFE_DEPLOYMENT.md` | 20 min |
| Field reference | `charts/klask/IMMUTABLE_FIELDS.md` | 30 min |
| Something broke | `charts/klask/SAFE_DEPLOYMENT.md` → Troubleshooting | 10 min |
| Understand changes | `HELM_SAFETY_IMPROVEMENTS.md` | 15 min |

### Reading Order

1. **Start here** → `charts/klask/README_IMMUTABLE_FIELDS.md` (Quick reference)
2. **Before deploying** → `charts/klask/SAFE_DEPLOYMENT.md` (Step-by-step guide)
3. **Need details** → `charts/klask/IMMUTABLE_FIELDS.md` (Field reference)
4. **Something wrong** → `charts/klask/SAFE_DEPLOYMENT.md` → Troubleshooting section

---

## 🛠️ Tools

### Validator Script

```bash
# Basic usage
./.claude/validators/helm-values-validator.sh my-values.yaml

# With custom namespace
./.claude/validators/helm-values-validator.sh -n production my-values.yaml

# With custom release name
./.claude/validators/helm-values-validator.sh -r my-release my-values.yaml

# Full options
./.claude/validators/helm-values-validator.sh \
  -n klask \
  -r klask \
  -k ~/.kube/test \
  my-values.yaml
```

### Deployment Script

```bash
# Standard deployment
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0

# Dry-run (no changes made)
DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0

# Skip backup
NO_BACKUP=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

---

## ✅ Deployment Checklist

Before every deployment, verify:

- [ ] Have you read the relevant documentation?
- [ ] Does your values file exist and is valid YAML?
- [ ] Have you validated with the validation script?
- [ ] Have you created a backup?
- [ ] Have you run a dry-run?
- [ ] Are you using the latest chart version?
- [ ] Is your kubeconfig correctly set?
- [ ] Are you deploying to the right namespace?
- [ ] Is Pod Security Standards `restricted` applied to the namespace?
  ```bash
  kubectl get ns klask -o jsonpath='{.metadata.labels.pod-security\.kubernetes\.io/enforce}' --kubeconfig ~/.kube/test
  # Should output: restricted
  ```

---

## 🆘 Quick Troubleshooting

### "Forbidden: updates to statefulset spec"

**Cause:** Trying to change an immutable field

**Solution:**
```bash
# 1. Check what changed
./.claude/validators/helm-values-validator.sh -n klask my-values.yaml

# 2. Revert the immutable field change
# OR delete and recreate:
kubectl delete statefulset klask-postgresql --cascade=orphan -n klask --kubeconfig ~/.kube/test
sleep 5
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

### "Pod never reached ready state"

**Solution:**
```bash
# Check pod status
kubectl describe pod klask-backend-xxxxx -n klask --kubeconfig ~/.kube/test

# Check logs
kubectl logs klask-backend-xxxxx -n klask --kubeconfig ~/.kube/test

# Common issues:
# - Database unreachable: Check connection string
# - Image pull failure: Check registry credentials
# - Resource limits: Check cluster capacity
```

### "StatefulSet rollout timeout"

**Solution:**
```bash
# Try again with longer timeout
helm upgrade klask oci://ghcr.io/klask-dev/klask:0.2.0 \
  -f my-values.yaml \
  --namespace klask \
  --timeout 10m
```

For more detailed troubleshooting, see `charts/klask/SAFE_DEPLOYMENT.md`.

---

## 📞 Getting Help

1. **For field questions** → Read `IMMUTABLE_FIELDS.md`
2. **For deployment issues** → Read `SAFE_DEPLOYMENT.md` → Troubleshooting
3. **For quick answers** → Read `README_IMMUTABLE_FIELDS.md` → FAQ
4. **For overview** → Read `HELM_SAFETY_IMPROVEMENTS.md`

---

## 🎯 Key Principles

### Safety First

- Always validate before deploying
- Always backup before major changes
- Always test in lower environments first
- Always use dry-run before actual deployment

### Know Your Fields

- Some fields are **immutable** (cannot change)
- Some fields are **mutable** (can change freely)
- Storage size can only **increase**, never decrease
- Database credentials are **immutable**

### Use the Tools

- Run the **validator script** before every deployment
- Use the **deployment script** for automation
- Create **backups** automatically
- Verify **health** after deployment

---

## Example: Complete Upgrade Workflow

```bash
#!/bin/bash
set -e

# Configuration
NAMESPACE="klask"
VALUES_FILE="my-values.yaml"
NEW_VERSION="0.2.1"

echo "🚀 Klask Helm Upgrade Workflow"
echo "==============================="
echo ""

# Step 1: Prepare
echo "Step 1: Preparing..."
echo "  Updating values file..."
sed -i "s/tag: .*/tag: \"$NEW_VERSION\"/" "$VALUES_FILE"
echo "  ✓ Values file updated"
echo ""

# Step 2: Validate
echo "Step 2: Validating..."
./.claude/validators/helm-values-validator.sh -n "$NAMESPACE" "$VALUES_FILE"
echo "  ✓ Validation passed"
echo ""

# Step 3: Dry-run
echo "Step 3: Dry-run..."
DRY_RUN=true ./scripts/safe-helm-deploy.sh "$NAMESPACE" "$VALUES_FILE" "$NEW_VERSION"
echo "  ✓ Dry-run successful"
echo ""

# Step 4: Confirm
read -p "Step 4: Proceed with deployment? (yes/no) " response
if [[ ! $response =~ ^yes$ ]]; then
  echo "Aborted."
  exit 0
fi
echo ""

# Step 5: Deploy
echo "Step 5: Deploying..."
./scripts/safe-helm-deploy.sh "$NAMESPACE" "$VALUES_FILE" "$NEW_VERSION"
echo "  ✓ Deployment complete"
echo ""

echo "✅ Upgrade complete!"
```

Save as `scripts/upgrade.sh` and run:

```bash
chmod +x scripts/upgrade.sh
./scripts/upgrade.sh
```

---

## 📖 File Structure

```
.
├── charts/klask/
│   ├── IMMUTABLE_FIELDS.md               ← Detailed reference
│   ├── SAFE_DEPLOYMENT.md                ← Step-by-step guide
│   ├── README_IMMUTABLE_FIELDS.md         ← Quick reference
│   ├── values.yaml                       ← Updated with warnings
│   ├── values-external-db.yaml           ← Enhanced example
│   └── ...
│
├── .claude/validators/
│   └── helm-values-validator.sh          ← Validation script
│
├── scripts/
│   └── safe-helm-deploy.sh               ← Deployment script
│
├── HELM_DEPLOYMENT_GUIDE.md              ← This file
├── HELM_SAFETY_IMPROVEMENTS.md           ← Overview of changes
└── ...
```

---

## 🎓 Learning Path

1. **5 minutes** - Read quick reference: `README_IMMUTABLE_FIELDS.md`
2. **10 minutes** - Understand the tools: This guide
3. **20 minutes** - Read deployment guide: `SAFE_DEPLOYMENT.md`
4. **30 minutes** - Study field reference: `IMMUTABLE_FIELDS.md`
5. **Practice** - Deploy using the scripts

---

## Version Info

- Chart Version: 0.2.0+
- Kubernetes: 1.18+
- Helm: 3.0+
- PostgreSQL: 15+

---

**Need help?** Check the [Quick Reference Guide](./charts/klask/README_IMMUTABLE_FIELDS.md)

---

*Last updated: 2025-10-27*
