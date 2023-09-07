workspaces := "./framework ./modules ./app-template"
modules := "./modules/contracts/apps/croncat ./modules/contracts/apps/dca ./modules/contracts/adapters/dex ./modules/contracts/adapters/cw-staking"

docs-install:
  cargo install mdbook
  cargo install mdbook-mermaid
  cargo install mdbook-admonish

# Pull a specific repo from its main remote
pull repo:
    git subtree pull --prefix={{repo}} {{repo}} main

# Push the local repo to a specific branch
push repo branch:
    git subtree pull --prefix={{repo}} {{repo}} {{branch}}

# Run a cargo command in all the workspace repos
cargo-all *command:
  #!/usr/bin/env bash
  set -e;
  for path in {{workspaces}}
  do 
    (cd $path; cargo {{command}}); 
  done
  set +e

check path:
    (cd {{path}}; cargo check)

check-all path:
    (cd {{path}}; cargo check --all-features)

nightly-build:
  just cargo-all build --all-features

# Wasms all the workspaces that can be wasm'd
wasm-all:
  ./scripts/wasm-all.sh

# Wasms all the workspaces that can be wasm'd
wasm-all-ci:
  ./scripts/wasm-all-ci.sh

schema-modules:
  #!/usr/bin/env bash
  set -e;
  for path in {{modules}}
  do 
    (cd $path; cargo run --example schema --features schema); 
  done
  set +e
