#!/usr/bin/env bash

# Add the CI to this repo
git remote add ci https://github.com/AbstractSDK/rust-ci
git fetch ci
git merge ci/main --allow-unrelated-histories

mv example.env .env

# Delete this script after running
rm -- "$0"
