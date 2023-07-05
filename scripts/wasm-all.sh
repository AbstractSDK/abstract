#!/usr/bin/env bash

# Detect the architecture #
if [[ $(arch) == "arm64" ]]; then
image="cosmwasm/rust-optimizer-arm64"
workspace_image="cosmwasm/workspace-optimizer-arm64"
abstract_image="abstractsdk/workspace-optimizer-arm64"
else
image="cosmwasm/rust-optimizer"
workspace_image="cosmwasm/workspace-optimizer"
abstract_image="abstractmoney/workspace-optimizer"
fi

current_dir=$(pwd)

# echo "Wasming app-template"
# cd ./app-template

# # Delete all the current wasms first
# rm -rf ./artifacts/*.wasm
# # Optimized builds
# docker run --rm -v "$(pwd)":/code \
# --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
# --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
# ${image}:0.12.13

# cd $current_dir


# echo "Wasming apps"
# cd ./apps

# # Delete all the current wasms first
# rm -rf ./artifacts/*.wasm
# # Optimized builds
# docker run --rm -v "$(pwd)":/code \
# -v "$(dirname "$(pwd)")/framework":/framework \
# -v "$(dirname "$(pwd)")/adapters":/adapters \
# -v "$(dirname "$(pwd)")/integrations":/integrations \
# -v "$(dirname "$(pwd)")/integration-bundles":/integration-bundles \
# --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
# --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
# ${workspace_image}:0.12.13

# cd $current_dir

echo "Wasming adapters"
cd ./adapters

# Delete all the current wasms first
rm -rf ./artifacts/*.wasm
# Optimized builds
docker run --rm -v "$(pwd)":/code \
-v "$(dirname "$(pwd)")/framework":/framework \
-v "$(dirname "$(pwd)")/apps":/apps \
-v "$(dirname "$(pwd)")/integrations":/integrations \
-v "$(dirname "$(pwd)")/adapter-packages":/adapter-packages \
-v "$(dirname "$(pwd)")/integration-bundles":/integration-bundles \
--mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
--mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
${abstract_image}:0.12.14

cd $current_dir
