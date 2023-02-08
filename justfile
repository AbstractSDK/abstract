build:
  cargo build

# Test everything
test:
  cargo nextest run

format:
  cargo fmt --all

lint:
  cargo clippy --all -- -D warnings
#  cargo clippy --all --all-targets --all-features -- -D warnings

lintfix:
  cargo clippy --fix --allow-staged --allow-dirty

refresh:
  cargo clean && cargo update

check-codecov:
  cat codecov.yml | curl --data-binary @- https://codecov.io/validate

publish:
  ./publish/publish.sh

wasm:
  ./publish/wasms.sh

wasm-module module:
  RUSTFLAGS='-C link-arg=-s' cargo wasm --package {{module}}

#wasm chain_name:
#  RUSTFLAGS='-C link-arg=-s' cargo ws exec --no-bail cargo wasm
#  if [[ {{chain}} == "terra" ]]; then RUSTFLAGS='-C link-arg=-s' cargo wasm --package dex --features terra --no-default-features; fi

full-deploy chain:
  (cd scripts && cargo run --bin full_deploy -- --network-id {{chain}})

publish-schemas version:
  SCHEMA_OUT_DIR=$(cd ../schemas && echo "$PWD") \
  VERSION={{version}} \
    cargo ws exec --no-bail bash -lc 'cargo schema && { outdir="$SCHEMA_OUT_DIR/abstract/${PWD##*/}/$VERSION"; mkdir -p "$outdir"; rm -rf "schema/raw"; cp -a "schema/." "$outdir"; }'