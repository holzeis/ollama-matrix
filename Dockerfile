FROM ubuntu:24.04

ARG BINARY=target/release/ollama-matrix

RUN apt-get update && \
    apt-get install -y \
    sqlite3 \
    ca-certificates \
    libssl3 \
    libgcc-s1 \
    libc6 && \
    rm -rf /var/lib/apt/lists/*

COPY ${BINARY} /usr/local/bin/ollama-matrix

RUN chmod +x /usr/local/bin/ollama-matrix && \
    ldd /usr/local/bin/ollama-matrix

USER 1000

ENTRYPOINT ["/usr/local/bin/ollama-matrix"]