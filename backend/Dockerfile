# ── Stage 1: Build ───────────────────────────────────────────────────
FROM rust:1.89-bookworm AS builder

# Install system libraries required by native crates
RUN apt-get update && apt-get install -y --no-install-recommends \
    clang llvm-dev libclang-dev \
    cmake pkg-config \
    libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

# Copy backend files
COPY backend/Cargo.toml backend/Cargo.lock ./

# Create a dummy main.rs so `cargo build` can resolve deps
RUN mkdir src && echo 'fn main() {}' > src/main.rs
RUN cargo build --release && rm -rf src target/release/.fingerprint/open_notebook_lm-*

# Copy the actual source
COPY backend/src ./src
RUN cargo build --release

# ── Stage 2: Runtime ────────────────────────────────────────────────
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*

RUN useradd --create-home --shell /bin/bash appuser

WORKDIR /app

# Copy the compiled binary
COPY --from=builder /app/target/release/open_notebook_lm ./

# Copy runtime assets
COPY backend/prompts ./prompts

# Create uploads directory
RUN mkdir -p uploads && chown appuser:appuser uploads

USER appuser

EXPOSE 8080

ENV RUST_LOG=info

HEALTHCHECK --interval=30s --timeout=3s \
  CMD curl -f http://localhost:8080/ || exit 1

CMD ["./open_notebook_lm"]
