#!/bin/bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Or use cargo workspaces
# https://crates.io/crates/cargo-workspaces

function print_usage() {
  echo "Usage: $0 [-h|--help] <new_version>"
  echo "e.g.: $0 0.8.0"
}

if [ "$#" -ne 1 ] || [ "$1" = "-h" ] || [ "$1" = "--help" ]
then
    print_usage
    exit 1
fi

# Check repo
SCRIPT_DIR="$(realpath "$(dirname "$0")")"
if [[ "$(realpath "$SCRIPT_DIR/..")" != "$(pwd)" ]]; then
  echo "Script must be called from the repo root"
  exit 2
fi

# Ensure repo is not dirty
CHANGES_IN_REPO=$(git status --porcelain --untracked-files=no)
if [[ -n "$CHANGES_IN_REPO" ]]; then
    echo "Repository is dirty. Showing 'git status' and 'git --no-pager diff' for debugging now:"
    git status && git --no-pager diff
    exit 3
fi

NEW="$1"
OLD=$(sed -n -e 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' packages/abstract-os/Cargo.toml)
echo "Updating old version $OLD to new version $NEW ..."

FILES_MODIFIED=()

for package_dir in packages/*/; do
  CARGO_TOML="$package_dir/Cargo.toml"
  sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

# Token
CARGO_TOML="contracts/abstract-token/Cargo.toml"
sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
FILES_MODIFIED+=("$CARGO_TOML")

# Testing
CARGO_TOML="contracts/testing/Cargo.toml"
sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
FILES_MODIFIED+=("$CARGO_TOML")

# Core
for contract_dir in contracts/core/*/; do
  CARGO_TOML="$contract_dir/Cargo.toml"
  sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

# extensions
for contract_dir in contracts/modules/extensions/*/; do
  CARGO_TOML="$contract_dir/Cargo.toml"
  sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

# apps
for contract_dir in contracts/modules/apps/*/; do
  CARGO_TOML="$contract_dir/Cargo.toml"
  sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

# services
for contract_dir in contracts/modules/services/*/; do
  CARGO_TOML="$contract_dir/Cargo.toml"
  sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

# native
for contract_dir in contracts/native/*/; do
  CARGO_TOML="$contract_dir/Cargo.toml"
  sed -i -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
  FILES_MODIFIED+=("$CARGO_TOML")
done

cargo build
FILES_MODIFIED+=("Cargo.lock")

echo "Staging ${FILES_MODIFIED[*]} ..."
git add "${FILES_MODIFIED[@]}"
git commit -m "Set version: $NEW"