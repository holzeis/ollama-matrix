FROM ubuntu:24.10

ARG BINARY=target/release/ollama-matrix

USER 1000

COPY $BINARY /usr/bin/ollama-matrix

ENTRYPOINT ["/usr/bin/ollama-matrix"]
