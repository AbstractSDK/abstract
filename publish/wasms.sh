#!/usr/bin/env bash
set -o errexit -o nounset -o pipefail
command -v shellcheck >/dev/null && shellcheck "$0"

NATIVE_CONTRACTS="ans-host version-control os-factory module-factory"
CORE_CONTRACTS="manager proxy"
MODULES="apis/dex apis/tendermint-staking apps/etf"


#for pack in $NATIVE_CONTRACTS; do
#  (
#    cd "contracts/native/$pack"
#    echo "Wasming $pack"
#    RUSTFLAGS='-C link-arg=-s' cargo wasm
#  )
#done

#for pack in $CORE_CONTRACTS; do
#  (
#    cd "contracts/core/$pack"
#    echo "Wasming $pack"
#    RUSTFLAGS='-C link-arg=-s' cargo wasm
#  )
#done


for pack in $MODULES; do
  (
    cd "contracts/modules/$pack"
    echo "Wasming $pack"
    RUSTFLAGS='-C link-arg=-s' cargo wasm
  )
done
