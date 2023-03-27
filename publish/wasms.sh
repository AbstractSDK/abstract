#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

NATIVE_CONTRACTS="ans-host version-control account-factory module-factory"
ACCOUNT_CONTRACTS="manager proxy"

for pack in $NATIVE_CONTRACTS; do
  (
    cd "contracts/native/$pack"
    echo "Wasming $pack"
    RUSTFLAGS='-C link-arg=-s' cargo wasm
  )
done

for pack in $ACCOUNT_CONTRACTS; do
  (
    cd "contracts/account/$pack"
    echo "Wasming $pack"
    RUSTFLAGS='-C link-arg=-s' cargo wasm
  )
done
