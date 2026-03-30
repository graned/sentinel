# ── Stage 1: build ────────────────────────────────────────────────────────────
FROM rust:1.91-slim AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq-dev \
    pkg-config \
    libssl-dev \
    curl \
    mold \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Pre-download Swagger UI so the utoipa build script never hits the network
# during CI / air-gapped builds. Keep version in sync with Dockerfile.dev.
RUN curl -fsSL -o /usr/local/swagger-ui.zip \
    "https://github.com/swagger-api/swagger-ui/archive/refs/tags/v5.17.14.zip"

ENV SWAGGER_UI_DOWNLOAD_URL="file:///usr/local/swagger-ui.zip"

# ── Dependency caching layer ──────────────────────────────────────────────────
# Copy only the workspace manifest files first so that dependency compilation
# is cached in a separate layer that survives application source changes.
COPY Cargo.toml Cargo.lock ./
COPY apps/sentinel-core/Cargo.toml ./apps/sentinel-core/
COPY packages/sentinel-policy-engine/Cargo.toml ./packages/sentinel-policy-engine/

# Stub source files so cargo can resolve and compile all dependencies before
# the real source arrives.
RUN mkdir -p apps/sentinel-core/src packages/sentinel-policy-engine/src \
    && echo 'fn main() {}' > apps/sentinel-core/src/main.rs \
    && touch packages/sentinel-policy-engine/src/lib.rs \
    && cargo build --release --manifest-path apps/sentinel-core/Cargo.toml \
    && rm -rf apps/sentinel-core/src packages/sentinel-policy-engine/src

# ── Application build ─────────────────────────────────────────────────────────
COPY apps/sentinel-core ./apps/sentinel-core
COPY packages/sentinel-policy-engine ./packages/sentinel-policy-engine

RUN cargo build --release --manifest-path apps/sentinel-core/Cargo.toml

# ── Stage 2: runtime ──────────────────────────────────────────────────────────
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    libpq5 \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/sentinel-core ./sentinel-core

# APP_HOST / APP_PORT default to 0.0.0.0 / 8000 inside the binary.
# Override via environment variables in the compose file if needed.
EXPOSE 8000

CMD ["./sentinel-core"]
