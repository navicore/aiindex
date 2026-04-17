# Run all CI checks (same as GitHub Actions!)
# This is what developers should run before pushing.
ci: fmt-check lint test build
    @echo "Safe to push to GitHub - CI will pass."

fmt:
    cargo fmt --all

fmt-check:
    cargo fmt --all -- --check

lint: lint-rust lint-pwa

lint-rust:
    cargo clippy --locked --workspace --all-targets -- -D warnings

lint-pwa:
    cd pwa && npm ci && npm run build

test:
    cargo test --locked --workspace --all-targets

build: build-server build-pwa

build-server:
    cargo build --locked --release

build-pwa:
    cd pwa && npm ci && npm run build

dev-server:
    RUST_LOG=info cargo run

dev-pwa:
    cd pwa && npm run dev

docker-build:
    docker build -t aiindex .

docker-run:
    docker run --rm -e FINNHUB_API_KEY -p 8080:8080 aiindex

clean:
    cargo clean
    rm -rf pwa/node_modules pwa/dist
