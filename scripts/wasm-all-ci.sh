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

# Delete the current artifacts folder.
rm -rf ./artifacts

# see https://github.com/CosmWasm/cw-plus/blob/main/.circleci/config.yml

# create a dummy container which will hold a volume with config
docker create -v /code --name with_code alpine /bin/true
# copy a config file into this volume
docker cp Cargo.toml with_code:/code
docker cp Cargo.lock with_code:/code

# copy code into this volume
docker cp ./contracts with_code:/code
docker cp ./packages with_code:/code
docker cp ./scripts with_code:/code

docker run --volumes-from with_code ${abstract_image}:0.14.0
docker cp with_code:/code/artifacts ./artifacts

cd $starting_dir