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
cargo-all command:
    for path in {{workspaces}}; do (cd $path; cargo {{command}} --all-features); done || exit 1

check path:
    (cd {{path}}; cargo check)

check-all path:
    (cd {{path}}; cargo check --all-features)
