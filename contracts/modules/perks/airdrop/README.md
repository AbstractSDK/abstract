# Airdrop

The Airdrop contract is for WHALE tokens airdrop claim during the intital protocol launch.

## Contract Design

### Handle Messages

| Message                               | Description                                                                                                                         |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------- |
| `ExecuteMsg::Claim`                   | Executes an airdrop claim for the Terra User.                                                                                       |
| `ExecuteMsg::TransferUnclaimedTokens` | Admin function. Transfers unclaimed WHALE tokens available with the contract to the recepient address once the claim window is over |
| `ExecuteMsg::UpdateConfig`            | Admin function to update any of the configuration parameters.                                                                       |

### Query Messages

| Message                    | Description                                                                                           |
| -------------------------- | ----------------------------------------------------------------------------------------------------- |
| `QueryMsg::Config`         | Returns the config info                                                                               |
| `QueryMsg::State`          | Returns the contract's state info                                                                     |
| `QueryMsg::HasUserClaimed` | Returns a boolean value indicating if the corresponding address have yet claimed their airdrop or not |
| `QueryMsg::UserInfo`       | Returns user's airdrop claim state                                                                    |

## How to Guide :: Get merkle proofs

### Create distribution lists for users

claimees_data.json

```
{[ { address: 'terra1k0jntykt7e4g3y88ltc60czgjuqdy4c9ax8tx2',
    amount: '43454523323'
  },
  { address: 'terra1xzlgeyuuyqje79ma6vllregprkmgwgavjx2h6m',
    amount: '1343252443'
  }
]}
```

### Get proof with user input

```
    import  {Terra_Merkle_Tree}  from "./helpers/terra_merkle_tree.js";

    const terra_merkle_tree = new Terra_Merkle_Tree(terra_claimees_data);
    const terra_tree_root = terra_merkle_tree.getMerkleRoot();

    let merkle_proof_for_terra_user_ = terra_merkle_tree.getMerkleProof({  "address":"terra1k0jntykt7e4g3y88ltc60czgjuqdy4c9ax8tx2",
                                                                            "amount": (43454523323).toString()
                                                                        } );

    console.log("Terra Merkle Root ", terra_tree_root)
    console.log("Terra Merkle Proof ", merkle_proof_for_terra_user_)
    console.log("Verify Terra Merkle Proof ", terra_merkle_tree.verify({  "address":"terra1k0jntykt7e4g3y88ltc60czgjuqdy4c9ax8tx2",
                                                                            "amount": (43454523323).toString()
                                                                        }) )
```

### Dependencies

- Rust v1.44.1+
- `wasm32-unknown-unknown` target
- Docker
- [LocalTerra](https://github.com/terra-project/LocalTerra)
- Node.js v16

### Envrionment Setup

1. Install `rustup` via https://rustup.rs/

2. Add `wasm32-unknown-unknown` target

```sh
rustup default stable
rustup target add wasm32-unknown-unknown
```

3. Install Node libraries required for testing:

```bash
cd scripts
npm install
```

## Build schema and run unit-tests

```
cargo schema
cargo test
```

### Compile

Make sure the current working directory is set to the root directory of this repository, then

```bash
cargo build
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="$(basename "$(pwd)")_cache",target=/code/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.11.3
```

### Test

Start LocalTerra:

```bash
cd /path/to/LocalTerra
git checkout main  # main branch for columbus-5 envrionment
git pull
docker-compose up
```

Run test scripts: inside `scripts` folder,

```bash
cd scripts

node --experimental-json-modules --loader ts-node/esm airdrop.spec.ts
```

## License

TBD
