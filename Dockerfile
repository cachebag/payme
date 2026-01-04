FROM rustlang/rust:nightly-bookworm AS backend-builder
WORKDIR /build
COPY backend/Cargo.toml backend/Cargo.lock ./
COPY backend/src ./src
RUN cargo build --release

FROM node:22-bookworm AS frontend-builder
WORKDIR /build
COPY frontend/package*.json ./
RUN npm ci
COPY frontend/ ./
RUN npm run build

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libsqlite3-0 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=backend-builder /build/target/release/payme /usr/local/bin/payme
COPY --from=frontend-builder /build/dist ./static

ENV DATABASE_URL=sqlite:/data/payme.db?mode=rwc
ENV PORT=3001

EXPOSE 3001

VOLUME ["/data"]

CMD ["payme"]

