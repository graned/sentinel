FROM rust:1.91 as builder
WORKDIR /build
COPY . .
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y libssl3 ca-certificates
WORKDIR /app
COPY --from=builder /build/target/release/sentinel-core .
CMD ["./sentinel-core"]
