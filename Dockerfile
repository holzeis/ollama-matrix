FROM ubuntu:24.10

ARG BINARY=target/release/ollama-matrix

RUN apt-get update && \
    apt-get install sqlite3 -y && \
    apt-get install ca-certificates -y

USER 1000

COPY $BINARY /usr/bin/ollama-matrix

ENTRYPOINT ["/usr/bin/ollama-matrix"]
