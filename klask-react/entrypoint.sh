#!/bin/sh

# Runtime environment variable replacement for Vite apps
# This allows changing API URLs without rebuilding the image

echo "🔧 Configuring frontend at runtime..."

# URL of the backend as seen by the browser.
# - unset: http://localhost:3000 (the backend port published by docker compose)
# - empty: same origin; the browser calls /api on the host that serves the
#   interface, which must route it to the backend (ingress, gateway, or the
#   nginx of this container, see BACKEND_UPSTREAM below)
API_BASE_URL=${BACKEND_BASE_URL-"http://localhost:3000"}

echo "📝 Setting API_BASE_URL to: '$API_BASE_URL'"

# Create a runtime config file that will be injected
# Escape and JSON-encode the API_BASE_URL to prevent injection attacks
if command -v jq >/dev/null 2>&1; then
    # Use jq for proper JSON encoding
    API_URL_JSON=$(jq -n --arg v "${API_BASE_URL}" '$v')
else
    # Fallback: manual escaping for environments without jq
    API_URL_JSON="\"$(echo "${API_BASE_URL}" | sed 's/\\/\\\\/g; s/"/\\"/g')\""
fi

cat > /usr/share/nginx/html/runtime-config.js << EOF
window.RUNTIME_CONFIG = {
  VITE_API_BASE_URL: ${API_URL_JSON}
};
EOF

echo "✅ Runtime configuration complete"

# Add nonce attribute to scripts in index.html for CSP support
# The nonce is generated per-request in nginx, this adds the attribute placeholder
sed -i 's|<script|<script nonce="__CSP_NONCE__"|g' /usr/share/nginx/html/index.html

echo "✅ CSP nonce attributes added"

# Backend address used by nginx to relay /api and /mcp (host:port, as seen from
# this container). nginx resolves it at request time with the resolvers of the
# container, which do not apply search domains: in Kubernetes use the full
# service name (<service>.<namespace>.svc.cluster.local:3000).
BACKEND_UPSTREAM=${BACKEND_UPSTREAM:-"klask-backend:3000"}
case "$BACKEND_UPSTREAM" in
    *[!A-Za-z0-9._:-]*|"")
        echo "❌ Invalid BACKEND_UPSTREAM: '$BACKEND_UPSTREAM' (expected host:port)" >&2
        exit 1
        ;;
esac

# DNS servers of the container (NGINX_RESOLVERS overrides them)
if [ -z "${NGINX_RESOLVERS:-}" ]; then
    NGINX_RESOLVERS=$(awk '$1 == "nameserver" && $2 !~ /%/ { if ($2 ~ /:/) printf "[%s] ", $2; else printf "%s ", $2 }' /etc/resolv.conf 2>/dev/null)
fi
NGINX_RESOLVERS=${NGINX_RESOLVERS:-"127.0.0.11"}

{
    echo "resolver ${NGINX_RESOLVERS% } valid=10s;"
    echo "set \$backend \"${BACKEND_UPSTREAM}\";"
} > /tmp/klask-nginx-backend.conf

echo "✅ Backend upstream for /api and /mcp: $BACKEND_UPSTREAM"

# Start nginx
exec nginx -g "daemon off;"
