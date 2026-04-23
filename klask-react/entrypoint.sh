#!/bin/sh

# Runtime environment variable replacement for Vite apps
# This allows changing API URLs without rebuilding the image

echo "🔧 Configuring frontend at runtime..."

# Default values
API_BASE_URL=${BACKEND_BASE_URL:-"http://localhost:3000"}

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

# Start nginx
exec nginx -g "daemon off;"
