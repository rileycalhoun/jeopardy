FROM rust:1.95

WORKDIR /app

RUN apt-get update && apt-get install -y \
    sqlite3 \
    libsqlite3-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

RUN cargo install cargo-watch --locked

COPY Cargo.toml Cargo.lock* ./

RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build

COPY . .

EXPOSE 3000

CMD ["cargo", "watch", "-x", "run"]
