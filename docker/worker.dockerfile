FROM rust:1.69 as builder
WORKDIR /usr/src/halo2_distributed
COPY . .
WORKDIR /usr/src/halo2_distributed/worker
RUN cargo install --path .


FROM debian:bullseye-slim
RUN apt-get update && apt-get install && rm -rf /var/lib/apt/lists/*
ENV WORKER=${WORKER}
COPY --from=builder /usr/local/cargo/bin/worker /usr/local/bin/worker
CMD worker ${WORKER}
