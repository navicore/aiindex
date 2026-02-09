# Stage 1: Build PWA
FROM node:22-slim AS pwa-build
WORKDIR /app/pwa
COPY pwa/package.json pwa/package-lock.json* ./
RUN npm ci
COPY pwa/ ./
RUN npm run build

# Stage 2: Build Rust server
FROM rust:1.85-bookworm AS server-build
WORKDIR /app
COPY Cargo.toml Cargo.lock* ./
COPY server/ server/
COPY stocks.toml .
RUN cargo build --release

# Stage 3: Runtime
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=server-build /app/target/release/aiindex-server /app/aiindex-server
COPY --from=pwa-build /app/pwa/dist /app/dist
COPY stocks.toml /app/stocks.toml

ENV BIND_ADDR=0.0.0.0:8080
ENV RUST_LOG=info
EXPOSE 8080

CMD ["/app/aiindex-server"]
