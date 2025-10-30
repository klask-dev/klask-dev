#!/bin/sh

# Runtime environment variable replacement for Vite apps
# This allows changing API URLs without rebuilding the image

echo "🔧 Configuring frontend at runtime..."

# Default values
API_BASE_URL=${BACKEND_BASE_URL:-"http://localhost:3000"}

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
