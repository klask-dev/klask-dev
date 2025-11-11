# Multi-stage build for Rust backend
FROM rust:slim-trixie as builder

# Install system dependencies (no more openssl/perl needed with rustls!)
RUN apt-get update && apt-get install -y \
    pkg-config \
    libpq-dev \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy Cargo.toml and Cargo.lock
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs

# Build dependencies (this layer will be cached unless Cargo.toml changes)
ENV SQLX_OFFLINE=true
RUN cargo build --release && rm src/main.rs

# Copy source code and sqlx cache
COPY src/ ./src/
COPY migrations/ ./migrations/
# COPY .sqlx/ ./.sqlx/

# Build the application
ENV SQLX_OFFLINE=true
RUN touch src/main.rs && cargo build --release

# Runtime stage
FROM debian:trixie-slim

# Install runtime dependencies including gosu for user switching
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    gosu \
    && rm -rf /var/lib/apt/lists/*

# Create app user with specific UID
RUN useradd -r -s /bin/false -u 1000 klask

# Set working directory
WORKDIR /app

# Copy the binary from builder stage
COPY --from=builder /app/target/release/klask-rs /app/klask-rs

# Copy migrations
COPY --from=builder /app/migrations /app/migrations

# Copy entrypoint script
COPY docker-entrypoint.sh /usr/local/bin/
RUN chmod +x /usr/local/bin/docker-entrypoint.sh

# Create directories for data
RUN mkdir -p /app/data /app/index && \
    chown -R klask:klask /app

# Don't switch to non-root user yet - entrypoint will do it
# USER klask  # Commented out - entrypoint handles user switching

# Expose port
EXPOSE 3000

# Set environment variables
ENV RUST_LOG=info
ENV PORT=3000
ENV HOST=0.0.0.0

# Health check - test if port 3000 is listening using /dev/tcp
HEALTHCHECK --interval=30s --timeout=3s --start-period=30s --retries=3 \
  CMD timeout 2 bash -c '</dev/tcp/localhost/3000' > /dev/null 2>&1 || exit 1

# Set entrypoint
ENTRYPOINT ["docker-entrypoint.sh"]

# Command to run the application
CMD ["./klask-rs"]