# Install the tools that are used in this justfile
install-tools:
  cargo install cargo-nextest --locked || true
  cargo install taplo-cli --locked || true
  cargo install cargo-watch || true
  cargo install cargo-limit || true

## Development Helpers ##

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

juno-local:
  docker kill juno_node_1 || true
  docker volume rm -f junod_data || true
  docker run --rm -d \
    --name juno_node_1 \
    -p 1317:1317 \
    -p 26656:26656 \
    -p 26657:26657 \
    -p 9090:9090 \
    -e STAKE_TOKEN=ujunox \
    -e UNSAFE_CORS=true \
    --mount type=volume,source=junod_data,target=/root \
    ghcr.io/cosmoscontracts/juno:15.0.0 \
    ./setup_and_run.sh juno16g2rahf5846rxzp3fwlswy08fz8ccuwk03k57y # You can add used sender addresses here

wasm:
  #!/usr/bin/env bash

  # Delete all the current wasms first
  rm -rf ./artifacts/*.wasm

  if [[ $(arch) == "arm64" ]]; then
    image="cosmwasm/rust-optimizer-arm64"
  else
    image="cosmwasm/rust-optimizer"
  fi

  # Optimized builds
  docker run --rm -v "$(pwd)":/code \
    --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
    --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
    ${image}:0.14.0

## Frontend Helpers ##

# Generate the typescript client for the app contract
ts-codegen: schema
  (cd packages/typescript && npm install && npm run codegen)

# Publish the typescript sdk
ts-publish: ts-codegen
  (cd packages/typescript && npm publish --access public)

# Generate the schemas for the app contract
schema:
  cargo schema

# Generate the schemas for this app and publish them to the schemas repository for access in the Abstract frontend
publish-schemas namespace name version: schema
  #!/usr/bin/env bash
  set -euxo pipefail

  # Pre-run check for 'gh' CLI tool
  if ! command -v gh &> /dev/null; then \
    echo "'gh' could not be found. Please install GitHub CLI."; exit; \
  fi

  # check that the metadata exists
  if [ ! -e "./metadata.json" ]; then \
    echo "Please create metadata.json for module metadata"; exit; \
  fi

  tmp_dir="$(mktemp -d)"
  schema_out_dir="$tmp_dir/{{namespace}}/{{name}}/{{version}}"
  metadata_out_dir="$tmp_dir/{{namespace}}/{{name}}"

  # Clone the repository to the temporary directory
  git clone https://github.com/AbstractSDK/schemas "$tmp_dir"

  # Create target directory structure and copy schemas
  mkdir -p "$schema_out_dir"
  cp -a "./schema/." "$schema_out_dir"

  # Copy metadata.json to the target directory
  cp "./metadata.json" "$metadata_out_dir"

  # Create a new branch with a name based on the inputs
  cd "$tmp_dir"
  git checkout -b '{{namespace}}/{{name}}/{{version}}'

  # Stage all new and changed files for commit
  git add .

  # Commit the changes with a message
  git commit -m 'Add schemas for {{namespace}} {{name}} {{version}}'

  # Create a pull request using 'gh' CLI tool
  gh pr create --title 'Add schemas for {{namespace}} {{name}} {{version}}' --body ""

## Exection commands ##

run-script script +CHAINS:
  cargo run --example {{script}} -- --network-ids {{CHAINS}}

publish +CHAINS:
  just run-script publish {{CHAINS}}
  