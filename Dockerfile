FROM rustlang/rust:nightly-trixie AS backend-builder
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release

FROM oven/bun:1.3 AS frontend-builder
WORKDIR /build
COPY frontend/package.json frontend/bun.lock ./
RUN bun install --frozen-lockfile
COPY frontend/ ./
RUN bun run build

FROM debian:trixie-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /build/target/release/payme /usr/local/bin/payme
COPY --from=frontend-builder /build/dist ./static

ENV DATABASE_URL=sqlite:/data/payme.db?mode=rwc
ENV PAYME_BIND=0.0.0.0:3001
ENV PAYME_STATIC_DIR=/app/static

EXPOSE 3001

VOLUME ["/data"]

CMD ["payme"]

