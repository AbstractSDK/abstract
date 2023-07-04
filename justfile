workspaces := "./framework ./adapters ./app-template ./integration-bundles"

docs-install:
  cargo install mdbook
  cargo install mdbook-mermaid
  cargo install mdbook-admonish

# Serve docs locally, pass --open to open in browser
docs-serve *FLAGS:
  (cd docs && mdbook serve {{FLAGS}}) 

docs-build:
  (cd docs && mdbook build)

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
  #!/usr/bin/env bash
  if [[ $(arch) == "arm64" ]]; then
    image="cosmwasm/rust-optimizer-arm64"
    workspace_image="cosmwasm/workspace-optimizer-arm64"
    abstract_image="abstractsdk/workspace-optimizer-arm64"
  else
    image="cosmwasm/rust-optimizer"
    workspace_image="cosmwasm/workspace-optimizer"
    abstract_image="abstractsdk/workspace-optimizer"
  fi

  current_dir=$(pwd)

  for path in ./app-template
  do 
    echo "Wasming $path"
    cd $path

    # Delete all the current wasms first
    rm -rf ./artifacts/*.wasm
    # Optimized builds
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      ${image}:0.12.13

    cd $current_dir
  done

  # TODO: add apps here once they compile
  for path in ./framework ./adapters
  do 
    echo "Wasming $path"
    cd $path

    # Delete all the current wasms first
    rm -rf ./artifacts/*.wasm
    # Optimized builds
    docker run --rm -v "$(pwd)":/code \
      --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
      --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
      ${workspace_image}:0.12.13

    cd $current_dir
  done