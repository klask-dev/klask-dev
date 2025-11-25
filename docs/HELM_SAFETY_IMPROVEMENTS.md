# Helm Chart Safety Improvements for Klask

## Summary

This document describes the improvements made to prevent "Forbidden: updates to statefulset spec" errors and provide safe deployment practices for the Klask Helm chart.

---

## Problem Statement

When upgrading Klask using Helm, users could encounter this error:

```
Error: UPGRADE FAILED: cannot patch "klask-postgresql" with kind StatefulSet:
StatefulSet.apps "klask-postgresql" is invalid: spec: Forbidden: updates to
statefulset spec for fields other than 'replicas', 'ordinals', 'template',
'updateStrategy', 'persistentVolumeClaimRetentionPolicy' and 'minReadySeconds'
are forbidden
```

This happens because Kubernetes StatefulSets have immutable fields that cannot be changed after creation. Users didn't have clear guidance on which fields are immutable and how to handle them.

---

## Solution Components

### 1. Documentation 📚

#### `charts/klask/IMMUTABLE_FIELDS.md`
**Comprehensive reference guide** for immutable fields.

**Contents:**
- Overview of immutable fields protection
- Complete list of immutable vs. mutable fields in Klask
- Why each field is immutable
- Safe upgrade practices
- Recovery procedures with step-by-step commands
- Summary table of all fields
- Best practices for PostgreSQL configuration

**Use case:** When you need to understand which fields can/cannot be changed

#### `charts/klask/SAFE_DEPLOYMENT.md`
**Step-by-step deployment guide** with detailed instructions.

**Contents:**
- Initial deployment walkthrough
- Pre-flight checklist for upgrades
- Safe upgrade procedure with dry-run
- Validation techniques
- Comprehensive troubleshooting section
- Emergency recovery options
- Automated deployment script examples

**Use case:** Before deploying or upgrading Klask

#### `charts/klask/README_IMMUTABLE_FIELDS.md`
**Quick reference** linking all resources together.

**Contents:**
- Quick start instructions
- Links to detailed documentation
- Common scenarios with examples
- FAQ and troubleshooting
- Summary table of resources

**Use case:** First point of reference for all Helm deployment questions

### 2. Validation Tools 🔧

#### `.claude/validators/helm-values-validator.sh`
**Automated validation script** that checks for immutable field violations.

**Features:**
- ✅ Detects existing StatefulSet configuration
- ✅ Compares against proposed values file
- ✅ Warns about immutable field changes
- ✅ Shows current vs. requested values
- ✅ Provides recovery recommendations
- ✅ Supports custom namespaces and release names

**Usage:**
```bash
# Basic validation
./.claude/validators/helm-values-validator.sh my-values.yaml

# With namespace
./.claude/validators/helm-values-validator.sh -n production my-values.yaml

# With custom release name
./.claude/validators/helm-values-validator.sh -r my-release my-values.yaml
```

**Output example:**
```
🔍 Helm StatefulSet Immutable Fields Validator

📋 Current StatefulSet configuration:
  serviceName: klask-postgresql-headless
  persistence.storageClass: default
  persistence.size: 2Gi

📊 Fields that CANNOT be modified:
  ✗ serviceName
  ✗ selector.matchLabels
  ✗ volumeClaimTemplates

⚙️  Fields that CAN be modified safely:
  ✓ template (pod spec)
  ✓ resources
  ✓ image tag
  ✓ persistence.size (only increase)
```

### 3. Deployment Automation 🚀

#### `scripts/safe-helm-deploy.sh`
**Guided deployment script** with integrated safety checks.

**Features:**
- ✅ Prerequisite validation (kubectl, helm, kubeconfig)
- ✅ Configuration validation (YAML syntax)
- ✅ Immutable field detection
- ✅ Automatic backup creation
- ✅ Dry-run with detailed manifests
- ✅ Manual confirmation before deployment
- ✅ Rollout monitoring
- ✅ Post-deployment verification
- ✅ Detailed logging and error messages

**Usage:**
```bash
# Standard deployment
./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0

# Dry-run first (no changes)
DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0

# Without backup
NO_BACKUP=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
```

**Workflow:**
1. ✓ Prerequisite checks
2. ✓ Values validation
3. ✓ Immutable field validation
4. ✓ Automatic backup
5. ✓ Dry-run with confirmation
6. ✓ Deployment execution
7. ✓ Health verification

### 4. Enhanced Configuration Documentation 📝

#### `charts/klask/values.yaml`
Updated with immutable field warnings.

**Changes:**
```yaml
nameOverride: ""
## ⚠️  IMMUTABLE: Changing this after initial deployment will cause StatefulSet upgrade failures!
## See: charts/klask/IMMUTABLE_FIELDS.md

postgresql:
  persistence:
    size: 2Gi
    ## ⚠️  IMMUTABLE AFTER FIRST DEPLOYMENT: Can only increase, never decrease

    storageClass: ""
    ## ⚠️  IMMUTABLE AFTER FIRST DEPLOYMENT: Cannot change between upgrades
    ## If you need a different storage class, recreate the StatefulSet
```

#### `charts/klask/values-external-db.yaml`
Completely redesigned with safety practices.

**Improvements:**
- ✅ Clear sections with color-coded fields (❌ mutable vs ✅ immutable)
- ✅ Usage instructions
- ✅ Best practices highlighted
- ✅ Pre-deployment checklist
- ✅ Security recommendations
- ✅ Examples for common scenarios
- ✅ Links to detailed documentation

---

## File Structure

```
klask-dev/
├── charts/klask/
│   ├── IMMUTABLE_FIELDS.md                # Detailed reference guide
│   ├── SAFE_DEPLOYMENT.md                 # Step-by-step deployment guide
│   ├── README_IMMUTABLE_FIELDS.md          # Quick reference
│   ├── values.yaml                        # Updated with warnings
│   ├── values-external-db.yaml            # Redesigned with safety practices
│   └── ... (other templates)
│
├── .claude/validators/
│   └── helm-values-validator.sh           # Validation script
│
├── scripts/
│   └── safe-helm-deploy.sh                # Automated deployment script
│
└── HELM_SAFETY_IMPROVEMENTS.md            # This file
```

---

## Usage Workflow

### For New Deployments

1. **Create your values file:**
   ```bash
   cp charts/klask/values.yaml my-values.yaml
   # Edit my-values.yaml with your configuration
   ```

2. **Validate:**
   ```bash
   ./.claude/validators/helm-values-validator.sh my-values.yaml
   ```

3. **Deploy:**
   ```bash
   ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
   ```

### For Upgrades

1. **Read the guide:**
   ```bash
   cat charts/klask/SAFE_DEPLOYMENT.md
   ```

2. **Check what's changing:**
   ```bash
   ./.claude/validators/helm-values-validator.sh -n klask new-values.yaml
   ```

3. **Deploy safely:**
   ```bash
   # Dry-run first
   DRY_RUN=true ./scripts/safe-helm-deploy.sh klask new-values.yaml 0.2.1

   # Then deploy
   ./scripts/safe-helm-deploy.sh klask new-values.yaml 0.2.1
   ```

### For Troubleshooting

1. **Check the FAQ:** `charts/klask/README_IMMUTABLE_FIELDS.md`
2. **Read immutable fields guide:** `charts/klask/IMMUTABLE_FIELDS.md`
3. **Follow recovery procedures:** `charts/klask/SAFE_DEPLOYMENT.md`

---

## Benefits

### For Users ✨

1. **Clear Guidance** - Know exactly which fields can/cannot be changed
2. **Error Prevention** - Validate before deploying, not after
3. **Safe Upgrades** - Automated checks and backups
4. **Quick Recovery** - Clear recovery procedures for mistakes
5. **Learning Resource** - Comprehensive documentation

### For Team ✨

1. **Consistency** - Standardized deployment process
2. **Reliability** - Validation prevents common mistakes
3. **Documentation** - Self-documenting Helm chart
4. **Automation** - Less manual work, fewer human errors
5. **Confidence** - Safe to upgrade without fear

### For Operations ✨

1. **Audit Trail** - Backups and logged changes
2. **Automation** - Scripts reduce manual work
3. **Monitoring** - Health checks after deployment
4. **Troubleshooting** - Comprehensive documentation
5. **Rollback** - Clear procedures for reverting changes

---

## Testing the Improvements

### Validator Script
```bash
cd /home/jeremie/git/github/klask-dev

# Test 1: No existing StatefulSet (should pass)
./.claude/validators/helm-values-validator.sh charts/klask/values.yaml

# Test 2: Create a values file with an immutable field change
cp charts/klask/values.yaml test-values.yaml
sed -i 's/nameOverride: ""/nameOverride: "test-klask"/' test-values.yaml

# Test 3: Validator should warn about the change
# (After StatefulSet is created)
```

### Deployment Script
```bash
# Test with dry-run
DRY_RUN=true ./scripts/safe-helm-deploy.sh klask-test charts/klask/values.yaml 0.2.0

# Test the validation step
# Should show:
# ✓ kubectl found
# ✓ helm found
# ✓ kubeconfig found
# ✓ Values file found
# ✓ Values file is valid YAML
# ✓ Immutable fields validation passed
```

---

## Integration with Existing Workflows

### CI/CD Integration

Add validation to CI/CD pipeline:

```yaml
# GitHub Actions / GitLab CI example
test_helm:
  script:
    - ./.claude/validators/helm-values-validator.sh values-prod.yaml
    - helm lint charts/klask
    - helm template klask charts/klask -f values-prod.yaml | kubeval
```

### Pre-commit Hook

Validate Helm files before committing:

```bash
#!/bin/bash
# .git/hooks/pre-commit
if [[ $(git diff --cached charts/klask/values*.yaml) ]]; then
  ./.claude/validators/helm-values-validator.sh charts/klask/values.yaml
fi
```

### Documentation Links

Reference in README:

```markdown
## Deployment

For detailed information on safe deployment practices, see:
- [Immutable Fields Reference](charts/klask/IMMUTABLE_FIELDS.md)
- [Safe Deployment Guide](charts/klask/SAFE_DEPLOYMENT.md)
- [Quick Reference](charts/klask/README_IMMUTABLE_FIELDS.md)
```

---

## Future Improvements

Possible enhancements:

1. **Interactive Deployment Wizard**
   - Step-by-step CLI prompts
   - Field validation as you type
   - Guided configuration

2. **Webhook Validation**
   - Pre-deployment API validation
   - Centralized policy enforcement
   - Audit logging

3. **Helm Chart Testing**
   - Automated chart testing with Helm Chart Testing (HCT)
   - Policy validation (OPA/Gatekeeper)
   - Dry-run in CI/CD

4. **Enhanced Monitoring**
   - Post-deployment health checks
   - Automatic rollback on health check failure
   - Deployment metrics and tracing

---

## Summary of Changes

| File | Change | Purpose |
|------|--------|---------|
| `charts/klask/IMMUTABLE_FIELDS.md` | NEW | Detailed reference guide |
| `charts/klask/SAFE_DEPLOYMENT.md` | NEW | Step-by-step deployment guide |
| `charts/klask/README_IMMUTABLE_FIELDS.md` | NEW | Quick reference |
| `charts/klask/values.yaml` | UPDATED | Added immutable field warnings |
| `charts/klask/values-external-db.yaml` | REDESIGNED | Enhanced with safety practices |
| `.claude/validators/helm-values-validator.sh` | NEW | Validation script |
| `scripts/safe-helm-deploy.sh` | NEW | Automated deployment script |
| `HELM_SAFETY_IMPROVEMENTS.md` | NEW | This summary document |

---

## Getting Started

1. **Read the quick reference:**
   ```bash
   cat charts/klask/README_IMMUTABLE_FIELDS.md
   ```

2. **Run the validator:**
   ```bash
   ./.claude/validators/helm-values-validator.sh my-values.yaml
   ```

3. **Use the deployment script:**
   ```bash
   ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
   ```

4. **Bookmark the full guide:**
   ```bash
   cat charts/klask/SAFE_DEPLOYMENT.md
   ```

---

## Questions?

Refer to the comprehensive documentation:

- **"Which fields can I change?"** → `IMMUTABLE_FIELDS.md`
- **"How do I deploy safely?"** → `SAFE_DEPLOYMENT.md`
- **"Something went wrong"** → `SAFE_DEPLOYMENT.md` → Troubleshooting
- **"Quick reference"** → `README_IMMUTABLE_FIELDS.md`

---

## Author Notes

These improvements are designed to:

1. **Prevent errors** - Catch issues before they happen
2. **Guide users** - Clear, step-by-step instructions
3. **Automate safely** - Scripts with built-in safeguards
4. **Document thoroughly** - Comprehensive reference material
5. **Enable confidence** - Users understand what's happening

By following the recommended practices, teams can deploy Klask reliably and safely, with minimal risk of StatefulSet immutable field errors.

---

*Last updated: 2025-10-27*
*All improvements integrated and tested*
