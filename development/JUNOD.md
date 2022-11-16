# Junod
Since we are deploying on the Juno network, we need to do a few steps for our workspace to be compatible with that.

## Installation
The most up-to-date version of junod as of writing this guide is`v6.0.0` so we will use that in this example.
```bash
git clone https://github.com/CosmosContracts/juno
cd juno
git fetch
git checkout v6.0.0
make install
```
This will install a local Juno daemon so that you can interact with a juno node. It will be located in your go-bin, so on a Mac that would look like: `/Users/adairrr/code/go-workspace/bin/junod`.
See [Junod Installation and setup - Junø](https://docs.junonetwork.io/validators/getting-setup) for more information.

## Deploying
### Compiling Contracts
Since we are in a monorepo, compiling the contracts to wasm is a bit more complicated than normal.

As normal, we want to run `cargo wasm`, which will generate some wasm files in the `artifacts` directory.

[CosmWasm/rust-optimizer](https://github.com/CosmWasm/rust-optimizer) is typically used to compile and "minify" contract code, and will have to be run at the *root* of this repo as follows:
```bash
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/workspace-optimizer:0.12.6
```
Please note that this will likely take a long time but at the end you should be presented with something like:
```
Optimizing ...
...
Optimizing version_control.wasm ...
Moving wasm files ...
Post-processing artifacts in workspace ...
85e8f6fe28caa805799cfdb9c65f937f348ea783fc5ae4f2ff7619635853b0f6  anchor.wasm
8e1e3ce3a03df1a14b5f2865845029ca0eefa705671f52d35ac4c504ea4d0d4f  manager.wasm
df5d5f633a59ee5db6735c4756b3190c7d4a48000a2bd0741d1ce3e90e68b647  ans_host.wasm
70b9cd6e1abc758063151ed265a181c3de847bed55cc5f8e71c3054eff879820  module_factory.wasm
5f3661f5a5530a485542296663267514c25a670f998d81c275dd72b96f9944d2  os_factory.wasm
506c5cb684b87623e074249315503a111923bcec7e80a8c7eb7409774b6f6047  pandora_dapp_base.wasm
7c59de7f02dd8628fbe4d40e1586f5c5f5766af7caff1dfa76dab8d0a1a1e714  proxy.wasm
156df0ca1078d9481227130d0f418a776288b1c0960bc4a3614311a2407a2eed  subscription.wasm
f4cb5520d4743a185170bcb8425bb61fe4c03c45f85cc45fc8752e001825b01a  terraswap_dapp.wasm
b6ee954dbec6e492008fbdc402af0bce6a288f933146dd977ff754a362d20ee2  vault_dapp.wasm
29ad539c70ae7400213cf3da5c3d8a81e368be9141a8279d48057d470f2edd11  version_control.wasm
done
```

### Instantiating the Contracts on juno
Once we have the addresses after compiling, we can take the ones that we want and instantiate the contracts on juno.
In the following examples we use the default LocalJuno key: `
```
cd artifacts
TX=$(junod tx wasm store cw_erc20.wasm  --from <your-key> --chain-id=<chain-id> --gas-prices 0.1ujunox --gas auto --gas-adjustment 1.3 -b block --output json -y | jq -r '.txhash')
CODE_ID=$(junod query tx $TX --output json | jq -r '.logs[0].events[-1].attributes[0].value')
```
Our deployment order is as follows:
1. Version Control
2. AnsHost


```bash
docker cp artifacts/version_control.wasm juno-node-1:/opt/version_control.wasm
TX=$(djuno tx wasm store version_control.wasm --from validator --chain-id=testing --gas-prices 0ujunox --gas auto --gas-adjustment 1.3 -b block --output json -y | jq -r '.txhash')
CODE_ID=$(junod query tx $TX --output json | jq -r '.logs[0].events[-1].attributes[0].value')
```
