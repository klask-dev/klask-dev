# Multi-stage build for the Rust backend
FROM rust:1.99.1-slim-trixie AS builder

# Accept build arguments for version info
ARG KLASK_VERSION=2.2.0
ARG GIT_COMMIT_HASH=""
ARG BUILD_TIMESTAMP=""

# Build-time system dependencies. TLS is provided by rustls and the database
# driver is pure Rust (sqlx), so no OpenSSL or libpq development package is needed.
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
ENV SQLX_OFFLINE=true
RUN cargo build --release && rm -rf src target/release/deps/klask_rs*

# Copy source code and migrations
COPY src/ ./src/
COPY migrations/ ./migrations/

# Build the application with version information
ENV KLASK_VERSION=${KLASK_VERSION}
ENV GIT_COMMIT_HASH=${GIT_COMMIT_HASH}
ENV BUILD_TIMESTAMP=${BUILD_TIMESTAMP}
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:trixie-slim

# Runtime dependencies:
# - ca-certificates: root store used by the rustls HTTP client (GitLab/GitHub crawling)
# - gosu: only used by docker-entrypoint.sh when the container is started as root
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    gosu \
    && rm -rf /var/lib/apt/lists/*

# Unprivileged application user with a fixed UID/GID (numeric so that
# Kubernetes can verify runAsNonRoot)
RUN groupadd --system --gid 1000 klask \
    && useradd --system --no-create-home --shell /usr/sbin/nologin --uid 1000 --gid klask klask

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/klask-rs /app/klask-rs

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod 0755 /usr/local/bin/docker-entrypoint.sh

# Create data directories owned by the application user. /app itself is owned
# by that user too (not its content: the binary and migrations stay read-only),
# because relative defaults such as ./models and ./vector-index are created there.
RUN mkdir -p /app/data /app/index \
    && chown 1000:1000 /app \
    && chown -R 1000:1000 /app/data /app/index

# Run as the unprivileged user by default
USER 1000:1000

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV PORT=3000
ENV HOST=0.0.0.0

# Health check: send a plain HTTP request to /health with bash's /dev/tcp
# (curl is not installed in the runtime image) and expect a 200 status line
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD ["bash", "-c", "exec 3<>/dev/tcp/127.0.0.1/${PORT:-3000} && printf 'GET /health HTTP/1.0\\r\\nHost: localhost\\r\\nConnection: close\\r\\n\\r\\n' >&3 && head -n 1 <&3 | grep -q ' 200 '"]

# Set entrypoint
ENTRYPOINT ["docker-entrypoint.sh"]

# Command to run the application
CMD ["./klask-rs"]
