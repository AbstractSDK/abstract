build:
  cargo build

# Test everything
test:
  cargo nextest run

format:
  cargo fmt --all
  find . -type f -iname "*.toml" -print0 | xargs -0 taplo format

lint:
  cargo clippy --all --all-features -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty --all-features
  just format

docs-install:
  cargo install mdbook
  cargo install mdbook-mermaid
  cargo install mdbook-admonish

# Serve docs locally, pass --open to open in browser
docs-serve *FLAGS:
  (cd docs && mdbook serve {{FLAGS}}) 

docs-build:
  (cd docs && mdbook build)

check:
  cargo check --all-features

refresh:
  cargo clean && cargo update

check-codecov:
  cat codecov.yml | curl --data-binary @- https://codecov.io/validate

# Publish crates
publish:
  ./publish/publish.sh

watch:
  cargo watch -x lcheck

watch-test:
  cargo watch -x "nextest run"

wasm:
  ./publish/wasms.sh

wasm-module module:
  RUSTFLAGS='-C link-arg=-s' cargo wasm --package {{module}}

run-script script +CHAINS:
  (cd scripts && cargo run --bin {{script}} -- --network-ids {{CHAINS}})

full-deploy +CHAINS:
  just run-script full_deploy {{CHAINS}}

publish-schemas version:
  SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
  VERSION={{version}} \
    cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; mkdir -p "$outdir"; rm -rf "schema/raw"; cp -a "schema/." "$outdir"; }'

# Download the wasms and deploy Abstract to all the chains
deploy-to-all-chains:
  just download-wasms
  just run-script full_deploy uni-6 pisco-1 juno-1 phoenix-1

download-wasms:
  (cd packages/abstract-interface && cargo run --example download_wasms)