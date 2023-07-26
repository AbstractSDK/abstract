#!/usr/bin/env bash

set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

function print_usage() {
  echo "Usage: $0 [-h|--help] version"
  echo "Yanks crates from crates.io with a specified version."
}

if [ $# -eq 0 ]; then
    echo "No arguments provided."
    print_usage
    exit 1
fi

if [ $# = 1 ] && { [ "$1" = "-h" ] || [ "$1" = "--help" ] ; }
then
    print_usage
    exit 1
fi

# Check if a version was supplied
if [ -z "$1" ]; then
  echo "You must supply a version number."
  print_usage
  exit 1
fi
# Get the version from the first argument
version=$1

BASE_PACKAGES="abstract-ica abstract-macros"
UTILS_PACKAGES="abstract-core abstract-testing abstract-sdk"
CORE_CONTRACTS="abstract-proxy abstract-manager"
NATIVE_CONTRACTS="abstract-ans-host abstract-account-factory abstract-module-factory abstract-version-control"
ALL_PACKAGES="abstract-interface abstract-adapter abstract-app abstract-ibc-host"

# list of all packages
all_packages=(
  $BASE_PACKAGES
  $UTILS_PACKAGES
  $CORE_CONTRACTS
  $NATIVE_CONTRACTS
  $ALL_PACKAGES
)

# Display the packages to be yanked
echo "The following packages will be yanked with version $version:"
printf '%s\n' "${all_packages[@]}"

# Ask for confirmation
read -p "Are you sure? [y/N] " -n 1 -r
echo
if [[ $REPLY =~ ^[Yy]$ ]]
then
  # loop over all packages and yank each one
  for package in ${all_packages[@]}; do
    cargo yank --vers $version $package
  done
fi
