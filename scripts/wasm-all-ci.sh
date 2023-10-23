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

# see https://github.com/CosmWasm/cw-plus/blob/main/.circleci/config.yml
echo "Wasming framework"
cd ./framework

# Delete the current artifacts folder.
# rm -rf ./artifacts

# Create lock file if it does not exist
if [ ! -f Cargo.lock ]; then
  cargo generate-lockfile
fi

# create a dummy container which will hold a volume with config
docker create -v /code --name with_code alpine /bin/true
# copy a config file into this volume
docker cp Cargo.toml with_code:/code
docker cp Cargo.lock with_code:/code
# copy code into this volume
docker cp ./contracts with_code:/code
docker cp ./packages with_code:/code
docker cp ./scripts with_code:/code
# Run the build
docker run --volumes-from with_code ${abstract_image}:0.15.0
# Copy the artifacts back out
docker cp with_code:/code/artifacts .
ls artifacts

cd $starting_dir

echo "Wasming modules"

docker rm -v modules_with_code
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
docker cp ./packages modules_with_code:/code
docker cp ./scripts modules_with_code:/code

# Run the build
docker run --volumes-from modules_with_code ${abstract_image}:0.15.0
# Copy the artifacts back out
docker cp modules_with_code:/code/artifacts .
ls artifacts
cd $starting_dir