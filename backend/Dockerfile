# ── Stage 1: Build ───────────────────────────────────────────────────
FROM rust:1.85-bookworm AS builder

# Install system libraries required by native crates
#   - clang / llvm         → RocksDB (surrealdb kv-rocksdb feature)
#   - cmake / pkg-config   → various -sys crates
#   - libssl-dev           → reqwest / rustls fallback
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang llvm-dev libclang-dev \
    cmake pkg-config \
    libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# ── Cache dependencies ──────────────────────────────────────────────
# Copy only manifests first to maximise Docker layer caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs so `cargo build` can resolve deps
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -rf src target/release/.fingerprint/open_notebook_lm-*

# ── Build the actual project ────────────────────────────────────────
COPY src ./src
RUN cargo build --release

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash appuser

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/open_notebook_lm ./

# Copy runtime assets
COPY prompts ./prompts

# Create uploads directory
RUN mkdir -p uploads && chown appuser:appuser uploads

USER appuser

# Default port (overridable via SERVER_PORT env var)
EXPOSE 8080

ENV RUST_LOG=info

CMD ["./open_notebook_lm"]
