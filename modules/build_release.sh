#!/usr/bin/env bash

# Delete all the current wasms first
rm -rf ./artifacts/*.wasm

if [[ $(arch) == "arm64" ]]; then
  image="cosmwasm/workspace-optimizer-arm64"
else
  image="cosmwasm/workspace-optimizer"
fi

# Optimized builds
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  -v "$(dirname "$(pwd)")/integrations":/integrations \
  -v "$(dirname "$(pwd)")/framework":/framework \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  ${image}:0.12.13