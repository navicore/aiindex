# Build everything
build: build-server build-pwa

# Build Rust server
build-server:
    cargo build --release

# Build PWA
build-pwa:
    cd pwa && npm ci && npm run build

# Run CI checks
ci: fmt-check clippy test build-pwa-strict

# Format check
fmt-check:
    cargo fmt -- --check

# Clippy lints
clippy:
    cargo clippy -- -D warnings

# Run tests
test:
    cargo test

# PWA strict build (fails on warnings)
build-pwa-strict:
    cd pwa && npm ci && npm run build

# Run server in development
dev-server:
    RUST_LOG=info cargo run

# Run PWA dev server
dev-pwa:
    cd pwa && npm run dev

# Docker build
docker-build:
    docker build -t aiindex .

# Docker run
docker-run:
    docker run --rm -e FINNHUB_API_KEY -p 8080:8080 aiindex

# Clean build artifacts
clean:
    cargo clean
    rm -rf pwa/node_modules pwa/dist
