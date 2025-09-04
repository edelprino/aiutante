FROM rust:latest AS builder
WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

ENV AIUTANTE_FOLDER=/agents
ENV AIUTANTE_API_HOST=0.0.0.0:3000
ENV RUST_LOG=aiutante=debug

COPY --from=builder /app/target/release/aiutante /usr/local/bin/aiutante
CMD ["aiutante", "api"]
