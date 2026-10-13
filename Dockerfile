FROM lukemathwalker/cargo-chef:latest-rust-1 AS chef
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    git \
    curl \
    xz-utils \
    unzip \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

FROM chef AS planner
COPY scripts/build_tigerbeetle_client.sh scripts/build_tigerbeetle_client.sh
RUN chmod +x scripts/build_tigerbeetle_client.sh && ./scripts/build_tigerbeetle_client.sh
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /app/third_party /app/third_party
COPY --from=planner /app/recipe.json recipe.json
# Build dependencies - this is the caching Docker layer!
RUN cargo chef cook --release --recipe-path recipe.json
# Build application
COPY . .
RUN cargo build --release --bin reputest --bin import_megajoules

# We do not need the Rust toolchain to run the binary!
FROM debian:bookworm-slim AS runtime
WORKDIR /app

# Install required system dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/reputest /usr/local/bin
COPY --from=builder /app/target/release/import_megajoules /usr/local/bin
ENTRYPOINT ["/usr/local/bin/reputest"]
