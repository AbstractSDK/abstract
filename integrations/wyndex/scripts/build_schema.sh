#!/bin/sh

set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

# Check repo
SCRIPT_DIR="$(realpath "$(dirname "$0")")"
if [[ "$(realpath "$SCRIPT_DIR/..")" != "$(pwd)" ]]; then
  echo "Script must be called from the repo root"
  exit 2
fi

for c in contracts/*/; do
  cd "$c"
  cargo schema
  cd -
done

mkdir -p schema
rm -f schema/*.json
cp contracts/*/schema/*.json schema/