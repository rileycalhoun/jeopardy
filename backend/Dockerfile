FROM rust:1.95-bookworm AS builder

WORKDIR /app

COPY Cargo.toml Cargo.lock ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release

COPY . .

RUN cargo build --release

FROM debian:bookworm-slim AS runtime

WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

COPY --from=builder /app/target/release/jeopardy-clone /usr/local/bin/jeopardy-clone
COPY --from=builder /app/question-packs ./question-packs
COPY --from=builder /app/migrations ./migrations

ENV BIND_ADDRESS=0.0.0.0 \
    BIND_PORT=8080 \
    QUESTION_PACK_DIR=/app/question-packs \
    RUST_LOG=info

EXPOSE 8080

CMD ["jeopardy-clone"]

FROM rust:1.95-bookworm AS migrator

WORKDIR /app

RUN cargo install sqlx-cli --version 0.8.6 --no-default-features --features rustls,postgres

COPY migrations ./migrations

ENTRYPOINT ["sqlx", "migrate", "run", "--source", "/app/migrations"]
