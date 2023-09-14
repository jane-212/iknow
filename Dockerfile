FROM rust as builder
WORKDIR /app
COPY src/ ./src
COPY Cargo.toml .
COPY build.rs .
COPY iknow.banner .
COPY .git/ ./.git
RUN cargo build --release

FROM debian
RUN apt-get update && apt-get install -y openssl && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/iknow .
COPY template/ ./template
CMD ["/app/iknow"]
