FROM rust:1.92.0-slim AS builder
ARG BINARY_NAME=ollama-matrix
WORKDIR /usr/src/$BINARY_NAME

# Install common build deps (adjust if your project needs others)
RUN apt-get update && \
    apt-get install -y --no-install-recommends pkg-config libssl-dev ca-certificates && \
    rm -rf /var/lib/apt/lists/*

# Cache dependency compilation: copy Cargo files and build a dummy main
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() { println!(\"compile deps\"); }" > src/main.rs
RUN cargo build --release || true

# Copy the full source and build the real binary
COPY . .
RUN cargo build --release -p ${BINARY_NAME}

# Final minimal runtime image
FROM debian:bookworm-slim
ARG BINARY_NAME=ollama-matrix

# Install runtime deps required by many Rust programs (OpenSSL libs, certificates)
RUN apt-get update && \
    apt-get install -y --no-install-recommends ca-certificates libssl3 && \
    rm -rf /var/lib/apt/lists/*

COPY --from=builder /usr/src/${BINARY_NAME}/target/release/${BINARY_NAME} /usr/local/bin/${BINARY_NAME}
RUN chmod +x /usr/local/bin/${BINARY_NAME}

# run as non-root user
USER 1000:1000
ENV RUST_LOG=info

# Use shell form so ARG can be respected at build-time when needed
ENTRYPOINT ["/bin/sh", "-lc", "exec /usr/local/bin/${BINARY_NAME}"]
