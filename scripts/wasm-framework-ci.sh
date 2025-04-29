#!/usr/bin/env bash

set -e

# Detect the architecture #
if [[ $(arch) == "arm64" ]]; then
  image="cosmwasm/optimizer-arm64:0.16.1"
else
  image="cosmwasm/optimizer:0.16.1"
fi

starting_dir=$(pwd)

# see https://github.com/CosmWasm/cw-plus/blob/main/.circleci/config.yml
echo "Wasming framework"
cd ./framework

# Remove for docker to successfuly copy
rm packages/abstract-interface/state.json
rm packages/abstract-interface/build.rs
rm packages/abstract-interface/artifacts || true

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
    cargo generate-lockfile
fi

docker rm -v with_code || true

# create a dummy container which will hold a volume with config
docker create -v /code --name with_code alpine /bin/true
# copy a config file into this volume
docker cp Cargo.toml with_code:/code
docker cp Cargo.lock with_code:/code
# copy code into this volume
docker cp ./workspace-hack with_code:/code
docker cp ./contracts with_code:/code
docker cp ./packages with_code:/code
# Run the build
docker run --volumes-from with_code ${image}
# Copy the artifacts back out
docker cp with_code:/code/artifacts/ .
ls artifacts