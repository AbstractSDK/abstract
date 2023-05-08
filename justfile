build:
  cargo build

# Test everything
test:
  cargo nextest run

format:
  cargo fmt --all

lint:
  cargo clippy --all -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty --all-features
  cargo fmt --all

refresh:
  cargo clean && cargo update

check-codecov:
  cat codecov.yml | curl --data-binary @- https://codecov.io/validate

tag:
  set -e
  git tag v`grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}'`
  git push origin v`grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}'`

watch:
  cargo watch -x lcheck

check:
  cargo check --all-features

watch-test:
  cargo watch -x "nextest run"

# `just wasm-contract etf--features export,terra --no-default-features`
wasm-contract module +args='':
  RUSTFLAGS='-C link-arg=-s' cargo wasm --package abstract-{{module}}-app {{args}}

# Wasm all the contracts in the repository for the given chain
wasm chain_name:
  just wasm-contract etf --features export --no-default-features
#  just wasm-contract subscription --features export --no-default-features

# Deploy a module to the chain
# ??? deploy-module module +args='': (wasm-module module)
# `just deploy-module dex pisco-1`
deploy-contract module network +args='':
  cargo deploy --package abstract-{{module}}-app -- --network-id {{network}} {{args}}

# Deploy all the apis
deploy network +args='':
  just wasm-contract etf
  just wasm-contract subscription
  just deploy-contract etf {{network}}
#  just deploy-contract subscription {{network}}

# Transfer the schemas to the Abstract schemas repo.
# TODO: git
publish-schemas version:
  SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
  VERSION={{version}} \
    cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; mkdir -p "$outdir"; rm -rf "schema/raw"; cp -a "schema/." "$outdir"; }'
