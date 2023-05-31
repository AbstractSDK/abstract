#!/usr/bin/env bash

# Add the CI to this repo
git remote add ci https://github.com/AbstractSDK/rust-ci
git fetch ci
git merge ci/main --squash

mv example.env .env

rm ./README.md
# Delete this script after running
rm -- "$0"
