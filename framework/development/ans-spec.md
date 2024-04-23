# ANS Convention Reference 1.0

## Table of Contents

- [Naming Convention Reference 1.0](#ans-convention-reference-10)
  - [Table of Contents](#table-of-contents)
  - [Definitions](#definitions)
  - [Delimiter Definitions](#delimiter-definitions)
    - [Delimiter Targets](#delimiter-targets)
    - [Delimiters](#delimiters)
    - [Delimiter Priorities](#delimiter-priorities)
  - [Naming Rules](#naming-rules)
  - [Items](#items)
    - [Asset](#asset)
    - [LP Token](#lp-token)
    - [Contract](#contract)
    - [Ibc Channel](#ibc-channel)
    - [Asset Pairs](#asset-pairs)
    - [Pool Metadata](#pool-metadata)
  - [Storage Maps](#storage-maps)

## Definitions

- **Protocol**: The blockchain or DApp protocol.
- **Asset**: Represents either native or wrapped tokens.
- **Attribute**: Additional information like pool type, additional protocol, etc.
- **Chain Name**: The name of the blockchain where the asset or contract exists.

## Delimiter Definitions

### Delimiter Targets

- **Asset**: Mapping for assets used inside an abstract namespace, e.g., `juno>juno` (mapping for `ujunox` inside the Juno chain).
- **Type**: Protocol type, e.g., `juno/croncat/factory` (chain protocol, app protocol, contract name).
- **Attribute**: Additional attribute or pool type, e.g., `junoswap/uusd,uust:stable` (chain protocol, assets, pool type).

### Delimiters

- `,`: Separates assets, e.g., `asset1,asset2`.
- `/`: Separates types, e.g., `contract_type/asset1,asset2`.
- `:`: Separates attributes, e.g., `abstract_namespace:contract`.
- `>`: Separates chain(s) and asset, e.g., `chain1>chain2>asset`.

### Delimiter Priorities

Delimiters should be processed in the following priority:

1. `:`
2. `/`
3. `,` and `>`

#### Example for Delimiter Priority

`juno:staking/wynd/juno>juno,juno>wynd`

1. `:` separates `juno` and `staking/wynd/juno>juno,juno>wynd`
2. `/` separates `staking` from `wynd/juno>juno,juno>wynd`
3. `,` and `>` separate assets and chains: `juno>juno` and `juno>wynd`

## Naming Rules

- All names must be in lowercase.
- Assets must be prefixed with the chain name, separated by `>`.

**Normalization**: If any rule is violated, the name will be considered invalid.

## Items

### Asset

- Key: `$CHAIN_NAME>$ASSET_NAME`
- Value: `{ "<native|cw20|cw1155>": "$ASSET_ADDRESS" }`

#### Examples and Notes

- **Example Key**: `terra2>luna`
- **Example Value**: `{ "native": "uluna" }`
- **Note**: All assets must be alphabetized.

### LP Token

- Key: `$PROTOCOL/$ASSET_A,$ASSET_B[,$ASSET_X+]`
- Value: `{ "<native|cw20|cw1155>": "$ASSET_ADDRESS" }`

### Contract

- Key: `$NAMESPACE:[$TYPE[/$ATTRIBUTE]]/$CONTRACT_NAME`
- Value: `$CONTRACT_ADDRESS`

#### Notes

- **Note**: Staking contracts will also belong here.

### Ibc Channel

- Key: `{ "connected_chain": "$HOST_CHAIN", "protocol": "$PROTOCOL" }`
- Value: `$CHANNEL_ID`

### Asset Pairs

- Key: `[$ASSET_A, $ASSET_B, $DEX_NAME]`
- Value: `[{ "unique_id": "$ABSTRACT_POOL_ID", "pool_address": { "<contract|id|separate_addresses>": $POOL_ADDRESS_ID_OR_SEPARATE_ADDRESSES } }]`

### Pool Metadata

- Key: `$ABSTRACT_POOL_ID`
- Value: `{ "dex": "$DEX", "pool_type": "<constant_product|stable|weighted|liquidity_bootstrap>", "assets": ["$ASSET_A", $ASSET_B] }`

## Storage Maps

These maps can be found in the updated [ans state documentation](../packages/abstract-std/src/native/ans_host.rs):

- [Asset](#asset) - `ASSET_ADDRESSES`
- [LP Token](#lp-token) - `ASSET_ADDRESSES`
- [Contract](#contract) - `CONTRACT_ADDRESSES`
- [Ibc Channel](#ibc-channel) - `CHANNELS`
- [Asset Pairs](#asset-pairs) - `ASSET_PAIRS`
- [Pool Metadata](#pool-metadata) - `POOL_METADATA`
