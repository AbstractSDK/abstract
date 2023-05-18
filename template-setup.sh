#!/usr/bin/env bash

git remote add ci https://github.com/AbstractSDK/rust-ci
git fetch ci
git merge ci/main --squash

# Delete this script after running
rm -- "$0"
