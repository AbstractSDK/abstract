#!/usr/bin/env bash
# shellcheck disable=all
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: ABSTRACT_TOKEN=[TOKEN] $0 [-h|--help]"
  echo -e "\tPublishes crates to crates.io."
  echo -e "\t- Set ABSTRACT_TOKEN variable for crates.io publisher token."
  echo -e "\t- Use it from the root of the monorepo."
  echo -e "\t- Make sure you don't have unstaged changes and you are on a branch for release."
}

publish_crate() {
  # Run the cargo publish command, capturing both stdout and stderr
  # Check if the command was successful
  if output=$(cargo publish --token $ABSTRACT_TOKEN 2>&1); then
    echo "Successfully published crate. 🎉"
  else
    # Check for the specific error message
    if [[ $output == *"crate version"*"is already uploaded"* ]]; then
      echo "Crate version is already uploaded 😱. Proceeding..."
    else
      echo "Failed to publish crate. Exiting. 😵"
      echo "Error: $output"
      return 1
    fi
  fi

  # Indicate success
  return 0
}

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 0
fi

# Make sure we're in the root
if [[ ! -f ".gitignore" ]]; then
  echo ".gitignore not found, make sure you are in the root of monorepo."
  exit 1
fi

# Check that crates.io token is set
if [ -z "${ABSTRACT_TOKEN}" ]; then
    echo "Must provide ABSTRACT_TOKEN for crates.io in environment" 1>&2
    exit 1
fi

# Make it stop if something goes wrong
set -o errexit -o nounset -o pipefail

# these are imported by other packages
BASE_PACKAGES="abstract-macros"
UTILS_PACKAGES="abstract-std abstract-testing abstract-sdk"
CORE_CONTRACTS="manager proxy"
NATIVE_CONTRACTS="ans-host account-factory module-factory version-control ibc-host ibc-client"

 for pack in $BASE_PACKAGES; do
   (
     cd "framework/packages/$pack"
     echo "Publishing base $pack"
    publish_crate
   )
 done

for pack in $UTILS_PACKAGES; do
  (
    cd "framework/packages/$pack"
    echo "Publishing util $pack"
    publish_crate
  )
done

for con in $CORE_CONTRACTS; do
  (
    cd "framework/contracts/account/$con"
    echo "Publishing account base $con"
    publish_crate
  )
done

for con in $NATIVE_CONTRACTS; do
  (
    cd "framework/contracts/native/$con"
    echo "Publishing native $con"
    publish_crate
  )
done

echo "All the contracts are published!"

# Now all the packages and standards

PACKAGES="abstract-interface abstract-adapter abstract-app abstract-standalone abstract-client"
STANDARDS="utils staking dex money-market"

for pack in $PACKAGES; do
  (
    cd "framework/packages/$pack"
    echo "Publishing $pack"
    publish_crate
  )
done

for pack in $STANDARDS; do
  (
    cd "framework/packages/standards/$pack"
    echo "Publishing $pack"
    publish_crate
  )
done

VERSION=$(grep -A1 "\[workspace.package\]" framework/Cargo.toml | awk -F'"' '/version/ {print $2}');
sh ./framework/publish/tag-release.sh "v$VERSION"
