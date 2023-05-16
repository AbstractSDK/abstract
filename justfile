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

install-docs:
  cargo install mdbook
  cargo install mdbook-mermaid

serve-docs:
  (cd docs && mdbook serve --open) 

build-docs:
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

run-script script chain:
  (cd scripts && cargo run --bin {{script}} -- --network-id {{chain}})

full-deploy chain:
  just run-script full_deploy {{chain}}

publish-schemas version:
  SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
  VERSION={{version}} \
    cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; mkdir -p "$outdir"; rm -rf "schema/raw"; cp -a "schema/." "$outdir"; }'
