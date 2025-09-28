FROM rust:latest AS builder
WORKDIR /app
COPY src src
COPY Cargo.toml Cargo.toml
COPY Cargo.lock Cargo.lock
RUN cargo build --release

FROM debian:bookworm-slim
# RUN apt-get update && apt-get install -y \
#     ca-certificates \
#     libssl3 \
#     && rm -rf /var/lib/apt/lists/*

# RUN (type -p wget >/dev/null || (apt update && apt install wget -y)) \
# 	&& mkdir -p -m 755 /etc/apt/keyrings \
# 	&& out=$(mktemp) && wget -nv -O$out https://cli.github.com/packages/githubcli-archive-keyring.gpg \
# 	&& cat $out | tee /etc/apt/keyrings/githubcli-archive-keyring.gpg > /dev/null \
# 	&& chmod go+r /etc/apt/keyrings/githubcli-archive-keyring.gpg \
# 	&& mkdir -p -m 755 /etc/apt/sources.list.d \
# 	&& echo "deb [arch=$(dpkg --print-architecture) signed-by=/etc/apt/keyrings/githubcli-archive-keyring.gpg] https://cli.github.com/packages stable main" | tee /etc/apt/sources.list.d/github-cli.list > /dev/null \
# 	&& apt update \
# 	&& apt install gh -y

# RUN apt-get -y install jq curl

ENV AIUTANTE_FOLDER=/agents
ENV AIUTANTE_API_HOST=0.0.0.0:3000
ENV RUST_LOG=aiutante=debug

COPY --from=builder /app/target/release/aiutante /usr/local/bin/aiutante
CMD ["aiutante", "api"]
