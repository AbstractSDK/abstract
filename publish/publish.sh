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

# these are imported by other packages
BASE_PACKAGES="abstract-ica abstract-os abstract-macros"
UTILS_PACKAGES="abstract-sdk"
CORE_CONTRACTS="proxy manager"
NATIVE_CONTRACTS="ans-host os-factory module-factory version-control"
ALL_PACKAGES="abstract-api abstract-app abstract-ibc-host abstract-boot"

# for pack in $BASE_PACKAGES; do
#   (
#     cd "packages/$pack"
#     echo "Publishing base $pack"
#     cargo publish
#   )
# done

# for pack in $UTILS_PACKAGES; do
#   (
#     cd "packages/$pack"
#     echo "Publishing util $pack"
#     cargo publish
#   )
# done

for con in $CORE_CONTRACTS; do
  (
    cd "contracts/core/$con"
    echo "Publishing core $con"
    cargo publish
  )
done

for con in $NATIVE_CONTRACTS; do
  (
    cd "contracts/native/$con"
    echo "Publishing native $con"
    cargo publish
  )
done

for pack in $ALL_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing $pack"
    cargo publish
  )
done

echo "Everything is published!"

VERSION=$(cat Cargo.toml | grep -m 1 version | sed 's/-/_/g' | grep -o '".*"' | sed 's/"//g');
git tag v$VERSION
git push origin v$VERSION
