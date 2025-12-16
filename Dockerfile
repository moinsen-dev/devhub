# DevHub - Multi-stage build
# Stage 1: Build Rust daemon
FROM rust:1.83-slim-bookworm AS rust-builder

WORKDIR /build

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust project files
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build release binary
RUN cargo build --release

# Stage 2: Build SvelteKit UI
FROM node:22-slim AS ui-builder

WORKDIR /build

# Copy UI project files
COPY devhub-ui/package*.json ./
RUN npm ci

COPY devhub-ui/ ./

# Build static site
RUN npm run build

# Stage 3: Final runtime image
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Copy built daemon binary
COPY --from=rust-builder /build/target/release/devhub /usr/local/bin/devhub

# Copy built UI (static files)
COPY --from=ui-builder /build/build ./ui

# Create data directories
RUN mkdir -p /data/registry /data/logs

# Environment variables
ENV DEVHUB_DATA_DIR=/data
ENV DEVHUB_UI_DIR=/app/ui
ENV DEVHUB_HOST=0.0.0.0
ENV DEVHUB_PORT=9876

# Expose ports
# 9876 = API daemon
EXPOSE 9876

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s \
    CMD curl -f http://localhost:9876/api/projects || exit 1

# Default command: run daemon
CMD ["devhub", "daemon"]
