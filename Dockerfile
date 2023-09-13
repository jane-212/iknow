FROM rust:latest as builder
WORKDIR /app
COPY src/ ./src
COPY Cargo.toml .
RUN cargo build --release

FROM debian:bullseye-slim
# RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/iknow .
CMD ["/app/iknow"]
