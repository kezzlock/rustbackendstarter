# ── Stage 1: Builder ──────────────────────────────────────────────────────────
FROM rust:1-slim AS builder

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    curl \
    unzip \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy manifests first for layer caching
COPY Cargo.toml Cargo.lock ./
# Create a dummy main to pre-compile deps
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release 2>/dev/null || true
RUN rm -rf src

# Copy actual source
COPY src ./src
COPY migrations ./migrations

# Build release binary
RUN touch src/main.rs && cargo build --release

# ── Stage 2: Runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy binary and migrations only — no source code in final image
COPY --from=builder /app/target/release/rustbackendstarter ./rustbackendstarter
COPY --from=builder /app/migrations ./migrations

# Create data directory for SQLite
RUN mkdir -p data

EXPOSE 3000

CMD ["./rustbackendstarter"]
