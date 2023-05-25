# Install the tools that are used in this justfile
install-tools:
  cargo install cargo-nextest --locked
  cargo install taplo-cli --locked
  cargo install cargo-watch
  cargo install cargo-limit

# Build everything
build:
  cargo build --all-features

# Test everything
test:
  cargo nextest run

watch-test:
  cargo watch -x "nextest run"

# Format your code and `Cargo.toml` files
fmt:
  cargo fmt --all
  find . -type f -iname "*.toml" -print0 | xargs -0 taplo format

lint:
  cargo clippy --all -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty --all-features
  just fmt

watch:
  cargo watch -x "lcheck --all-features"

check:
  cargo check --all-features

deploy:
  cargo run --example deploy --features 

wasm:
  docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.13