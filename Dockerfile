# LUMOS Docker Image
# Multi-stage build for minimal image size
#
# Usage:
#   docker run --rm -v $(pwd):/workspace ghcr.io/getlumos/lumos generate schema.lumos
#
# Build locally:
#   docker build -t lumos .
#   docker run --rm -v $(pwd):/workspace lumos generate schema.lumos

# =============================================================================
# Stage 1: Build
# =============================================================================
FROM rust:1.82-slim-bookworm AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy manifests first for better caching
COPY Cargo.toml Cargo.lock ./
COPY packages/core/Cargo.toml packages/core/
COPY packages/cli/Cargo.toml packages/cli/
COPY packages/lsp/Cargo.toml packages/lsp/
COPY packages/cargo-lumos/Cargo.toml packages/cargo-lumos/
COPY packages/npm/package.json packages/npm/

# Create dummy source files for dependency caching
RUN mkdir -p packages/core/src packages/cli/src packages/lsp/src packages/cargo-lumos/src packages/core/benches \
    && echo "fn main() {}" > packages/cli/src/main.rs \
    && echo "fn main() {}" > packages/lsp/src/main.rs \
    && echo "fn main() {}" > packages/cargo-lumos/src/main.rs \
    && echo "pub fn dummy() {}" > packages/core/src/lib.rs \
    && echo "fn main() {}" > packages/core/benches/benchmarks.rs \
    && echo "fn main() {}" > packages/core/benches/borsh_comparison.rs

# Build dependencies (cached layer)
RUN cargo build --release --package lumos-cli 2>/dev/null || true

# Copy actual source code (benches excluded via .dockerignore - use dummy stubs)
COPY packages/core/src packages/core/src
COPY packages/cli/src packages/cli/src
COPY packages/lsp/src packages/lsp/src
COPY packages/cargo-lumos/src packages/cargo-lumos/src

# Touch source files to invalidate cache and rebuild
RUN touch packages/core/src/lib.rs packages/cli/src/main.rs

# Build release binary
RUN cargo build --release --package lumos-cli

# Strip binary for smaller size
RUN strip /build/target/release/lumos

# =============================================================================
# Stage 2: Runtime
# =============================================================================
FROM debian:bookworm-slim

# Labels for GitHub Container Registry
LABEL org.opencontainers.image.source="https://github.com/getlumos/lumos"
LABEL org.opencontainers.image.description="LUMOS - Type-safe schema language for Solana development"
LABEL org.opencontainers.image.licenses="MIT OR Apache-2.0"

# Copy binary from builder
COPY --from=builder /build/target/release/lumos /usr/local/bin/lumos

# Create non-root user for security (prevents container escape attacks)
# Using UID 1000 for compatibility with most host systems
RUN groupadd -r -g 1000 lumos && \
    useradd -r -u 1000 -g lumos lumos

# Set working directory for mounted volumes
WORKDIR /workspace

# Ensure workspace is accessible (owner will be set by volume mount)
RUN chown lumos:lumos /workspace

# Switch to non-root user
USER lumos

# Default entrypoint
ENTRYPOINT ["lumos"]

# Default command (show help)
CMD ["--help"]
