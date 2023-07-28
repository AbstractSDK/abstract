#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help]"
  echo "Publishes crates to crates.io."
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

ALL_PACKAGES="abstract-interface abstract-adapter abstract-app abstract-ibc-host utils"
# These need to update the dependency of abstract-interface to use the version
OTHER_PACKAGES="dex staking"

for pack in $ALL_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish
  )
done

echo "Packages are published!"

read -p "Please update the version of 'abstract-interface' (deps & dev-deps) in the dex and stakingto the published version and type 'yes' to continue: " input
if [ "$input" != "yes" ]
then
  echo "The script will terminate now. Please run it again after updating the version."
  exit 1
fi

echo "Continuing with the publication of other packages..."

for pack in $OTHER_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish --allow-dirty
  )
done

echo "All packages have been published!"

# VERSION=$(grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}');
# git tag v"$VERSION"
# git push origin v"$VERSION"
