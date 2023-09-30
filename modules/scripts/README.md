# Abstract scripts
**alpha software**
A useful repo for interacting with abstract_core smart-contracts.

## Binaries
The binaries located in [`src/bin/`](src/bin/) are executable files that interact with an available blockchain node. 
Which node and at what address/port can be specified in the [`.env`](.env) file. 
You can copy clone an example env from [here](https://github.com/CyberHoward/cosm-rust-script/blob/main/cosm-script/example.env).

## Resources
The [`resources/`](resources/) directory contains addresses and asset information related to the different networks and deployments. 
default_chains.json holds the chain and deployment information.
so 

## Libraries
 [`libs/`](libs/) contains a helper library which helps in removing boilerplate from the scripts. It also contains an `interfaces` library which stores the interfaces to interact with the contracts. 

Interfaces for common contracts like cw-20s are available with [`cosm-rust-script`](https://github.com/CyberHoward/cosm-rust-script/tree/main/cw-plus-script).

## Usage
clone this repo
clone .env.example and rename to .env and change params to your needs. 

WASM_DIR: where your build .wasm artifacts are stored
STORE="default_chains.json"

then run the script 
` cargo run --bin native_contracts `
