build:
  cargo build

# Test everything
test:
  cargo nextest run

format:
  cargo fmt --all

lint:
  cargo clippy --all --all-features -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty --all-features
  cargo fmt --all

check:
  cargo check --all-features

refresh:
  cargo clean && cargo update

check-codecov:
  cat codecov.yml | curl --data-binary @- https://codecov.io/validate

watch:
  cargo watch -x lcheck

watch-test:
  cargo watch -x "nextest run"

# `just wasm-module cw-staking --features export,terra --no-default-features`
wasm-contract module +args='':
  RUSTFLAGS='-C link-arg=-s' cargo wasm --package abstract-{{module}} {{args}}

# Wasm all the contracts in the repository for the given chain
wasm chain_name:
  just wasm-contract cw-staking --features {{chain_name}},export --no-default-features
  just wasm-contract dex --features {{chain_name}},export --no-default-features
  just wasm-contract tendermint-staking
#  if [[ {{chain}} == "terra" ]]; then RUSTFLAGS='-C link-arg=-s' cargo wasm --package dex --features terra --no-default-features; fi

# Deploy a module to the chain
# ??? deploy-module module +args='': (wasm-module module)
# `just deploy-module dex pisco-1`
deploy-contract module network +args='':
  cargo deploy --package abstract-{{module}} -- --network-id {{network}} {{args}}

# Deploy all the apis
deploy network +args='':
  just deploy-contract dex {{network}}
  just deploy-contract cw-staking {{network}}
  just deploy-contract tendermint-staking {{network}}

publish-schemas version:
  SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
  VERSION={{version}} \
    cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; mkdir -p "$outdir"; rm -rf "schema/raw"; cp -a "schema/." "$outdir"; }'
    
publish-tag:
  set -e
  git tag v`grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}'`
  git push origin v`grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}'`
