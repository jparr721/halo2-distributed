FROM rust:1.69 as builder
WORKDIR /usr/src/worker
COPY ../worker .
RUN cargo install --path .

FROM debian:bullseye-slim
RUN apt-get update && apt-get install -y extra-runtime-dependencies && rm -rf /var/lib/apt/lists/*
COPY --from=builder /usr/local/cargo/bin/worker /usr/local/bin/worker
CMD ["worker"]
