#!/usr/bin/env bash

set -e

# Detect the architecture #
if [[ $(arch) == "arm64" ]]; then
  image="cosmwasm/optimizer-arm64:0.16.0"
else
  image="cosmwasm/optimizer:0.16.0"
fi

starting_dir=$(pwd)

# see https://github.com/CosmWasm/cw-plus/blob/main/.circleci/config.yml
echo "Wasming modules"

docker rm -v modules_with_code || true

# create a dummy container which will hold a volume with config
docker create -v /code -v /integrations -v /framework --name modules_with_code alpine /bin/true

# copy directories to container.
docker cp ./integrations modules_with_code:/
docker cp ./framework modules_with_code:/

# go into the directory we want to compile
cd ./modules

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
    cargo generate-lockfile
fi

# Delete the current artifacts folder.
# rm -rf ./artifacts
# copy a config file into this volume
docker cp Cargo.toml modules_with_code:/code
docker cp Cargo.lock modules_with_code:/code
# copy code into this volume
docker cp ./contracts modules_with_code:/code

# Run the build
docker run --volumes-from modules_with_code ${image}
# Copy the artifacts back out
docker cp modules_with_code:/code/artifacts/ .
ls artifacts