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
CORE_CONTRACTS="manager proxy"
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

read -p "Please comment out abstract-adapter and abstract-app in manager/Cargo.toml#dev-dependencies and type 'yes' to continue: " input
if [ "$input" != "yes" ]
then
  echo "The script will terminate now. Please run it again after updating the version."
  exit 1
fi

for con in $CORE_CONTRACTS; do
  (
    cd "contracts/account/$con"
    echo "Publishing account base $con"
    cargo publish --allow-dirty
  )
done

for con in $NATIVE_CONTRACTS; do
  (
    cd "contracts/native/$con"
    echo "Publishing native $con"
    cargo publish --allow-dirty
  )
done

echo "All the contracts are published!"
