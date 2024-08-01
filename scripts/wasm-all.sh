#!/usr/bin/env bash

# Detect the architecture #
if [[ $(arch) == "arm64" ]]; then
  image="cosmwasm/optimizer-arm64:0.16.0"
else
  image="cosmwasm/optimizer:0.16.0"
fi

starting_dir=$(pwd)

# echo "Wasming framework"
# cd ./framework

# # Delete all the current wasms first
# rm -rf ./artifacts/*.wasm
# # Optimized builds
# docker run --rm -v "$(pwd)":/code \
#   --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
#   --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
#   ${image}

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
  ${image}
