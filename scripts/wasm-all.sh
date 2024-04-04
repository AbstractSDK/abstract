#!/usr/bin/env bash

# Detect the architecture #
if [[ $(arch) == "arm64" ]]; then
image="cosmwasm/rust-optimizer-arm64"
workspace_image="cosmwasm/workspace-optimizer-arm64"
abstract_image="abstractmoney/workspace-optimizer-arm64"
else
image="cosmwasm/rust-optimizer"
workspace_image="cosmwasm/workspace-optimizer"
abstract_image="abstractmoney/workspace-optimizer"
fi

starting_dir=$(pwd)

echo "Wasming framework"
cd ./framework

# Delete all the current wasms first
rm -rf ./artifacts/*.wasm
# Optimized builds
docker run --rm -v "$(pwd)":/code \
--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
${workspace_image}:0.15.0

cd $starting_dir

echo "Wasming apps"
cd ./modules

# Delete all the current wasms first
rm -rf ./artifacts/*.wasm
# Optimized builds
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  -v "$(dirname "$(pwd)")/integrations":/integrations \
  -v "$(dirname "$(pwd)")/framework":/framework \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  ${abstract_image}:0.15.0
