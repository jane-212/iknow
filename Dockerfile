FROM rust as builder
WORKDIR /app
COPY src/ ./src
COPY Cargo.toml .
COPY build.rs .
COPY banner .
RUN cargo build --release

FROM debian
ENV TZ=Asia/Shanghai
RUN apt-get update && apt-get install -y openssl ca-certificates && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/iknow .
COPY template/ ./template
CMD ["/app/iknow"]
