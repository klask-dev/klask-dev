#!/bin/bash
# Validates that immutable StatefulSet fields are not being changed during Helm upgrades
# This prevents the "Forbidden: updates to statefulset spec" error
#
# Usage:
#   ./.claude/validators/helm-values-validator.sh ~/temp/custom-values.yml
#   ./.claude/validators/helm-values-validator.sh -n klask values-production.yaml

set -e

KUBECONFIG="${KUBECONFIG:-$HOME/.kube/test}"
NAMESPACE="${NAMESPACE:-klask}"
VALUES_FILE="${1:-.}"
RELEASE_NAME="klask"

# Colors for output
RED='\033[0;31m'
YELLOW='\033[1;33m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse command line arguments
while [[ $# -gt 0 ]]; do
  case $1 in
    -n|--namespace)
      NAMESPACE="$2"
      shift 2
      ;;
    -r|--release)
      RELEASE_NAME="$2"
      shift 2
      ;;
    -k|--kubeconfig)
      KUBECONFIG="$2"
      shift 2
      ;;
    *)
      VALUES_FILE="$1"
      shift
      ;;
  esac
done

echo -e "${BLUE}🔍 Helm StatefulSet Immutable Fields Validator${NC}\n"

# Check if StatefulSet exists
if ! kubectl get statefulset "${RELEASE_NAME}-postgresql" \
    --kubeconfig "$KUBECONFIG" \
    -n "$NAMESPACE" &>/dev/null 2>&1; then
  echo -e "${GREEN}✓ No existing StatefulSet found. Fresh deployment - all fields are safe to modify.${NC}"
  exit 0
fi

echo -e "${YELLOW}⚠️  Existing StatefulSet found. Checking for immutable field changes...${NC}\n"

# Get current StatefulSet configuration
echo "📋 Current StatefulSet configuration:"
echo ""

# 1. Check serviceName
CURRENT_SERVICE=$(kubectl get statefulset "${RELEASE_NAME}-postgresql" \
  --kubeconfig "$KUBECONFIG" \
  -n "$NAMESPACE" \
  -o jsonpath='{.spec.serviceName}')
echo -e "  ${BLUE}serviceName:${NC} $CURRENT_SERVICE"

# 2. Check selector labels
echo -e "  ${BLUE}selector.matchLabels:${NC}"
kubectl get statefulset "${RELEASE_NAME}-postgresql" \
  --kubeconfig "$KUBECONFIG" \
  -n "$NAMESPACE" \
  -o jsonpath='{.spec.selector.matchLabels}' | python3 -m json.tool 2>/dev/null | sed 's/^/    /'

# 3. Check storage class
CURRENT_STORAGE_CLASS=$(kubectl get statefulset "${RELEASE_NAME}-postgresql" \
  --kubeconfig "$KUBECONFIG" \
  -n "$NAMESPACE" \
  -o jsonpath='{.spec.volumeClaimTemplates[0].spec.storageClassName}' 2>/dev/null || echo "default")
echo -e "  ${BLUE}persistence.storageClass:${NC} $CURRENT_STORAGE_CLASS"

# 4. Check storage size
CURRENT_STORAGE_SIZE=$(kubectl get statefulset "${RELEASE_NAME}-postgresql" \
  --kubeconfig "$KUBECONFIG" \
  -n "$NAMESPACE" \
  -o jsonpath='{.spec.volumeClaimTemplates[0].spec.resources.requests.storage}' 2>/dev/null || echo "unknown")
echo -e "  ${BLUE}persistence.size:${NC} $CURRENT_STORAGE_SIZE"

echo ""
echo "📊 Fields that CANNOT be modified (Kubernetes protection):"
echo -e "  ${RED}✗ serviceName${NC}"
echo -e "  ${RED}✗ selector.matchLabels${NC}"
echo -e "  ${RED}✗ volumeClaimTemplates${NC}"
echo -e "  ${RED}✗ podManagementPolicy${NC}"
echo ""
echo "⚙️  Fields that CAN be modified safely:"
echo -e "  ${GREEN}✓ template (pod spec)${NC}"
echo -e "  ${GREEN}✓ replicas${NC}"
echo -e "  ${GREEN}✓ updateStrategy${NC}"
echo -e "  ${GREEN}✓ persistence.size${NC} (only increase)"
echo -e "  ${GREEN}✓ resources${NC}"
echo -e "  ${GREEN}✓ image tag${NC}"
echo ""

# If a values file is provided, do more detailed checking
if [ -f "$VALUES_FILE" ] && [[ "$VALUES_FILE" != "." ]]; then
  echo "🔎 Analyzing your values file..."
  echo ""

  # Extract PostgreSQL configuration from values file
  echo "Configuration in $VALUES_FILE:"

  # Check if nameOverride or fullnameOverride are present
  if grep -q "nameOverride\|fullnameOverride" "$VALUES_FILE"; then
    echo -e "  ${RED}⚠️  WARNING: nameOverride or fullnameOverride detected!${NC}"
    echo "     These affect the serviceName and cannot be changed."
  fi

  # Check PostgreSQL values
  if grep -A 20 "^postgresql:" "$VALUES_FILE" | grep -q "username\|database"; then
    echo -e "  ${RED}⚠️  WARNING: PostgreSQL auth values detected!${NC}"
    echo "     Changing username or database cannot be done via Helm upgrade."
  fi

  if grep -A 20 "^postgresql:" "$VALUES_FILE" | grep -q "storageClass"; then
    VALUES_STORAGE=$(grep -A 20 "^postgresql:" "$VALUES_FILE" | grep "storageClass:" | awk -F: '{print $2}' | tr -d ' "')
    if [ -n "$VALUES_STORAGE" ] && [ "$VALUES_STORAGE" != "$CURRENT_STORAGE_CLASS" ]; then
      echo -e "  ${RED}✗ CRITICAL: storageClass change detected!${NC}"
      echo "     Current: $CURRENT_STORAGE_CLASS"
      echo "     Requested: $VALUES_STORAGE"
      echo "     This will cause the upgrade to fail!"
    fi
  fi
fi

echo ""
echo "📚 Recommendations:"
echo "  1. Use existingSecret for immutable PostgreSQL credentials"
echo "  2. Never change nameOverride, fullnameOverride, or storage class"
echo "  3. To upgrade with breaking changes, use:"
echo "     kubectl delete statefulset ${RELEASE_NAME}-postgresql --cascade=orphan -n $NAMESPACE"
echo ""
