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
BASE_PACKAGES="abstract-ica abstract-macros"
UTILS_PACKAGES="abstract-core abstract-testing abstract-sdk"
CORE_CONTRACTS="proxy manager"
NATIVE_CONTRACTS="ans-host account-factory module-factory version-control"

 for pack in $BASE_PACKAGES; do
   (
     cd "packages/$pack"
     echo "Publishing base $pack"
     cargo publish
   )
 done

for pack in $UTILS_PACKAGES; do
  (
    cd "packages/$pack"
    echo "Publishing util $pack"
    cargo publish
  )
done

for con in $CORE_CONTRACTS; do
  (
    cd "contracts/account/$con"
    echo "Publishing account base $con"
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

echo "Everything is published!"

VERSION=$(grep -A1 "\[workspace.package\]" Cargo.toml | awk -F'"' '/version/ {print $2}');
git tag v"$VERSION"
git push origin v"$VERSION"
