#!/bin/sh

# Runtime environment variable replacement for Vite apps
# This allows changing API URLs without rebuilding the image

echo "🔧 Configuring frontend at runtime..."

# Handle BACKEND_BASE_URL:
# - If set to empty string: use relative paths (/api) for nginx proxy
# - If not set or has value: use that value
# - Default: http://localhost:3000 (for standalone mode)
if [ -z "${BACKEND_BASE_URL+x}" ]; then
    # Variable not set at all
    API_BASE_URL="http://localhost:3000"
elif [ -z "$BACKEND_BASE_URL" ]; then
    # Variable set but empty - use relative paths
    API_BASE_URL=""
else
    # Variable set with value - use it
    API_BASE_URL="$BACKEND_BASE_URL"
fi

echo "📝 Setting API_BASE_URL to: '$API_BASE_URL'"

# Create a runtime config file that will be injected
cat > /usr/share/nginx/html/runtime-config.js << EOF
window.RUNTIME_CONFIG = {
  VITE_API_BASE_URL: "$API_BASE_URL"
};
EOF

echo "✅ Runtime configuration complete"

# Start nginx
exec nginx -g "daemon off;"