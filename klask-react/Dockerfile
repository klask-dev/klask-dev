# Multi-stage build for the React frontend
FROM node:26.12-alpine AS builder

# Build arguments carrying version information into the bundle
ARG VITE_APP_VERSION=2.0.0
ARG VITE_APP_COMMIT=""
ARG VITE_APP_BUILD_TIME=""

# Set working directory
WORKDIR /app

# Copy package files first to cache the dependency layer
COPY package*.json ./

# Install all dependencies (devDependencies are needed to build).
# The project has no native addons: no compiler or image libraries are required.
RUN npm ci --no-audit --no-fund

# Copy source code
COPY . .

# Build the application with version information injected as environment variables
ENV VITE_APP_VERSION=${VITE_APP_VERSION}
ENV VITE_APP_COMMIT=${VITE_APP_COMMIT}
ENV VITE_APP_BUILD_TIME=${VITE_APP_BUILD_TIME}
RUN npm run build

# Production stage with nginx
FROM nginx:1.29-alpine

# Install jq for secure JSON encoding in entrypoint.sh
RUN apk add --no-cache jq

# Copy custom nginx config
COPY nginx.conf /etc/nginx/conf.d/default.conf
COPY nginx-proxy.inc /etc/nginx/klask-proxy.inc

# Copy entrypoint script
COPY entrypoint.sh /entrypoint.sh
RUN chmod +x /entrypoint.sh

# Copy built application from builder stage
COPY --from=builder /app/dist /usr/share/nginx/html

# Non-root user (numeric UID so Kubernetes can verify runAsNonRoot)
RUN addgroup -g 1001 -S nginx-app && \
    adduser -S nginx-app -u 1001 -G nginx-app

# Set proper permissions
RUN chown -R nginx-app:nginx-app /usr/share/nginx/html && \
    chown -R nginx-app:nginx-app /var/cache/nginx && \
    chown -R nginx-app:nginx-app /var/log/nginx && \
    chown -R nginx-app:nginx-app /etc/nginx/conf.d

# Create the pid file location for nginx
RUN touch /var/run/nginx.pid && \
    chown -R nginx-app:nginx-app /var/run/nginx.pid

# Switch to non-root user
USER 1001:1001

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=5s --start-period=15s --retries=3 \
  CMD wget --no-verbose --tries=1 --spider http://localhost:8080/ || exit 1

# Use entrypoint for runtime configuration
ENTRYPOINT ["/entrypoint.sh"]
