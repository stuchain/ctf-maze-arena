# syntax=docker/dockerfile:1

FROM rust:1.86-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
COPY migrations ./migrations
COPY benches ./benches
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/ctf-maze-arena /app/ctf-maze-arena
COPY --from=builder /app/migrations /app/migrations
RUN useradd -r -u 10001 -M -s /bin/false app \
    && mkdir -p /app/data \
    && chown -R app:app /app
USER app
ENV PORT=8080
EXPOSE 8080
CMD ["./ctf-maze-arena"]
