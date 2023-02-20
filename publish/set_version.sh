#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Or use cargo workspaces
# https://crates.io/crates/cargo-workspaces

function print_usage() {
  echo "Usage: $0 [-h|--help] <new_version>"
  echo "e.g.: $0 0.9.0"
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
OLD=$(sed -n -e 's/^version[[:space:]]*=[[:space:]]*"\(.*\)"/\1/p' Cargo.toml)
echo "Updating old version $OLD to new version $NEW ..."

FILES_MODIFIED=()

CARGO_TOML="packages/abstract-boot/Cargo.toml"
sed -i '' -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "$CARGO_TOML"
FILES_MODIFIED+=("$CARGO_TOML")

sed -i '' -e "s/version[[:space:]]*=[[:space:]]*\"$OLD\"/version = \"$NEW\"/" "Cargo.toml"
FILES_MODIFIED+=("Cargo.toml")


cargo build
FILES_MODIFIED+=("Cargo.lock")

echo "Staging ${FILES_MODIFIED[*]} ..."
# git add "${FILES_MODIFIED[@]}"
# git commit -m "Set version: $NEW"
