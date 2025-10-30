#!/bin/bash
# Safe Helm Deployment Script for Klask
# This script provides a step-by-step safe deployment process with validation,
# backup, dry-run, and verification.
#
# Usage:
#   ./scripts/safe-helm-deploy.sh [namespace] [values-file] [chart-version]
#
# Examples:
#   ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
#   ./scripts/safe-helm-deploy.sh production values-prod.yaml latest
#   DRY_RUN=true ./scripts/safe-helm-deploy.sh klask my-values.yaml 0.2.0
#
# Environment Variables:
#   DRY_RUN=true          - Run in dry-run mode (no actual deployment)
#   NO_BACKUP=true        - Skip backup creation
#   KUBECONFIG            - Path to kubeconfig file (default: ~/.kube/test)
#   HELM_TIMEOUT          - Helm timeout (default: 5m)

set -euo pipefail

# ============================================================================
# Configuration
# ============================================================================

NAMESPACE="${1:-klask}"
VALUES_FILE="${2:-}"
CHART_VERSION="${3:-latest}"
CHART="oci://ghcr.io/klask-dev/klask"
RELEASE_NAME="klask"

KUBECONFIG="${KUBECONFIG:-$HOME/.kube/test}"
HELM_TIMEOUT="${HELM_TIMEOUT:-5m}"
DRY_RUN="${DRY_RUN:-false}"
NO_BACKUP="${NO_BACKUP:-false}"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# ============================================================================
# Functions
# ============================================================================

print_header() {
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
  echo -e "${BLUE}$1${NC}"
  echo -e "${BLUE}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
}

print_step() {
  echo ""
  echo -e "${GREEN}✓${NC} $1"
}

print_error() {
  echo -e "${RED}✗${NC} $1"
}

print_warning() {
  echo -e "${YELLOW}⚠${NC}  $1"
}

print_info() {
  echo -e "${BLUE}ℹ${NC} $1"
}

confirm() {
  local prompt="$1"
  local response
  read -p "$(echo -e ${YELLOW}$prompt${NC}) (yes/no): " -r response
  [[ $response =~ ^[Yy]es$ ]]
}

# ============================================================================
# Validation
# ============================================================================

validate_prerequisites() {
  print_header "Prerequisites Check"

  # Check kubectl
  if ! command -v kubectl &> /dev/null; then
    print_error "kubectl not found. Please install kubectl."
    exit 1
  fi
  print_step "kubectl found: $(kubectl version --client --short 2>/dev/null)"

  # Check helm
  if ! command -v helm &> /dev/null; then
    print_error "helm not found. Please install helm."
    exit 1
  fi
  print_step "helm found: $(helm version --short 2>/dev/null)"

  # Check kubeconfig
  if [ ! -f "$KUBECONFIG" ]; then
    print_error "kubeconfig not found at $KUBECONFIG"
    exit 1
  fi
  print_step "kubeconfig found at $KUBECONFIG"

  # Check values file
  if [ -z "$VALUES_FILE" ] || [ ! -f "$VALUES_FILE" ]; then
    print_error "Values file not provided or not found"
    echo "Usage: $0 [namespace] [values-file] [chart-version]"
    echo "Example: $0 klask my-values.yaml 0.2.0"
    exit 1
  fi
  print_step "Values file found at $VALUES_FILE"

  # Check namespace exists or can be created
  if ! kubectl get namespace "$NAMESPACE" --kubeconfig "$KUBECONFIG" &>/dev/null; then
    print_warning "Namespace $NAMESPACE does not exist. It will be created."
  else
    print_step "Namespace $NAMESPACE exists"
  fi
}

validate_values_file() {
  print_header "Configuration Validation"

  print_info "Validating values file: $VALUES_FILE"

  # Check if the values file is valid YAML
  if ! yq eval . "$VALUES_FILE" > /dev/null 2>&1; then
    print_error "Values file is not valid YAML"
    exit 1
  fi
  print_step "Values file is valid YAML"

  # Run immutable fields validator if it exists
  VALIDATOR_SCRIPT="./.claude/validators/helm-values-validator.sh"
  if [ -f "$VALIDATOR_SCRIPT" ]; then
    print_info "Running immutable fields validator..."
    if bash "$VALIDATOR_SCRIPT" -n "$NAMESPACE" -k "$KUBECONFIG" "$VALUES_FILE"; then
      print_step "Immutable fields validation passed"
    else
      print_error "Immutable fields validation failed"
      exit 1
    fi
  else
    print_warning "Validator script not found at $VALIDATOR_SCRIPT (skipping)"
  fi
}

# ============================================================================
# Backup
# ============================================================================

create_backup() {
  if [ "$NO_BACKUP" = "true" ]; then
    print_warning "Backup skipped (NO_BACKUP=true)"
    return
  fi

  print_header "Creating Backup"

  BACKUP_DIR="/tmp/klask-backups"
  mkdir -p "$BACKUP_DIR"
  BACKUP_ID=$(date +%Y%m%d-%H%M%S)

  print_info "Backup ID: $BACKUP_ID"

  # Backup current values
  print_info "Backing up current Helm values..."
  if helm get values "$RELEASE_NAME" -n "$NAMESPACE" \
      --kubeconfig "$KUBECONFIG" > "$BACKUP_DIR/values-$BACKUP_ID.yaml" 2>/dev/null; then
    print_step "Helm values backed up to $BACKUP_DIR/values-$BACKUP_ID.yaml"
  else
    print_warning "Could not backup Helm values (release may not exist yet)"
  fi

  # Backup Kubernetes resources
  print_info "Backing up Kubernetes resources..."
  kubectl get statefulset,deployment,pvc,secret,service -n "$NAMESPACE" \
    --kubeconfig "$KUBECONFIG" \
    -o yaml > "$BACKUP_DIR/resources-$BACKUP_ID.yaml" 2>/dev/null || true
  print_step "Kubernetes resources backed up to $BACKUP_DIR/resources-$BACKUP_ID.yaml"

  # Save backup ID for later reference
  echo "$BACKUP_ID" > "$BACKUP_DIR/LAST_BACKUP"
  print_step "Backup complete"
}

# ============================================================================
# Dry-Run
# ============================================================================

run_dry_run() {
  print_header "Dry-Run Deployment"

  local full_chart="$CHART:$CHART_VERSION"
  print_info "Chart: $full_chart"
  print_info "Namespace: $NAMESPACE"
  print_info "Release: $RELEASE_NAME"

  # Create dry-run output
  DRY_RUN_FILE="/tmp/helm-dry-run-$(date +%s).yaml"
  print_info "Dry-run output: $DRY_RUN_FILE"

  if helm upgrade "$RELEASE_NAME" "$full_chart" \
      -f "$VALUES_FILE" \
      --namespace "$NAMESPACE" \
      --kubeconfig "$KUBECONFIG" \
      --install \
      --dry-run \
      --debug > "$DRY_RUN_FILE" 2>&1; then
    print_step "Dry-run successful"
    print_info "Review generated manifests at: $DRY_RUN_FILE"
  else
    print_error "Dry-run failed!"
    echo ""
    tail -50 "$DRY_RUN_FILE"
    exit 1
  fi
}

# ============================================================================
# Confirmation
# ============================================================================

confirm_deployment() {
  print_header "Deployment Confirmation"

  echo ""
  echo "Summary:"
  echo "  Release: $RELEASE_NAME"
  echo "  Namespace: $NAMESPACE"
  echo "  Chart: $CHART:$CHART_VERSION"
  echo "  Values: $VALUES_FILE"
  echo "  Kubeconfig: $KUBECONFIG"
  echo "  Timeout: $HELM_TIMEOUT"
  echo ""

  if ! confirm "Proceed with deployment?"; then
    print_warning "Deployment cancelled by user"
    exit 0
  fi
}

# ============================================================================
# Deployment
# ============================================================================

execute_deployment() {
  print_header "Executing Deployment"

  local full_chart="$CHART:$CHART_VERSION"

  print_info "Deploying $RELEASE_NAME from $full_chart..."
  echo ""

  if helm upgrade "$RELEASE_NAME" "$full_chart" \
      -f "$VALUES_FILE" \
      --namespace "$NAMESPACE" \
      --create-namespace \
      --kubeconfig "$KUBECONFIG" \
      --install \
      --wait \
      --timeout "$HELM_TIMEOUT"; then
    print_step "Deployment successful"
  else
    print_error "Deployment failed!"
    print_warning "Use 'helm rollback $RELEASE_NAME' to revert if needed"
    exit 1
  fi
}

# ============================================================================
# Verification
# ============================================================================

verify_deployment() {
  print_header "Deployment Verification"

  print_info "Checking pod status..."
  echo ""
  kubectl get pods -n "$NAMESPACE" --kubeconfig "$KUBECONFIG" || true
  echo ""

  print_info "Checking Helm release status..."
  echo ""
  helm status "$RELEASE_NAME" -n "$NAMESPACE" --kubeconfig "$KUBECONFIG" || true
  echo ""

  print_info "Checking services..."
  echo ""
  kubectl get svc -n "$NAMESPACE" --kubeconfig "$KUBECONFIG" || true
  echo ""

  print_step "Verification complete"
}

# ============================================================================
# Main Flow
# ============================================================================

main() {
  print_header "Safe Helm Deployment for Klask"

  echo ""
  echo "Configuration:"
  echo "  Namespace: $NAMESPACE"
  echo "  Values File: $VALUES_FILE"
  echo "  Chart Version: $CHART_VERSION"
  echo "  Release Name: $RELEASE_NAME"
  echo "  Kubeconfig: $KUBECONFIG"
  echo "  Dry-Run Mode: $DRY_RUN"
  echo ""

  # Step 1: Validate prerequisites
  validate_prerequisites

  # Step 2: Validate values
  validate_values_file

  # Step 3: Create backup
  create_backup

  # Step 4: Dry-run
  run_dry_run

  # Step 5: Handle dry-run mode
  if [ "$DRY_RUN" = "true" ]; then
    print_header "Dry-Run Complete"
    print_info "No changes were made to the cluster."
    print_info "Review the plan and run again without DRY_RUN=true to deploy."
    exit 0
  fi

  # Step 6: Confirmation
  confirm_deployment

  # Step 7: Deploy
  execute_deployment

  # Step 8: Verify
  verify_deployment

  # Final summary
  print_header "Deployment Complete!"
  echo ""
  echo -e "${GREEN}✓ Successfully deployed $RELEASE_NAME to $NAMESPACE${NC}"
  echo ""
  echo "Next steps:"
  echo "  • Monitor logs: kubectl logs -n $NAMESPACE -f -l app.kubernetes.io/name=klask-backend"
  echo "  • Port-forward: kubectl port-forward -n $NAMESPACE svc/klask-backend 3000:3000"
  echo "  • Rollback if needed: helm rollback $RELEASE_NAME -n $NAMESPACE"
  echo ""
}

# ============================================================================
# Execution
# ============================================================================

if [ "${BASH_SOURCE[0]}" = "${0}" ]; then
  main "$@"
fi
