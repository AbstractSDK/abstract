# Naming Convention Reference

## Table of Contents
- [Naming Convention Reference](#naming-convention-reference)
  - [Table of Contents](#table-of-contents)
  - [Delimiter Definitions](#delimiter-definitions)
    - [Delimiter targets](#delimiter-targets)
    - [Delimiters](#delimiters)
    - [Delimiter priorities](#delimiter-priorities)
  - [Naming rules](#naming-rules)
  - [Items](#items)
    - [Asset](#asset)
      - [Examples](#examples)
      - [Notes](#notes)
    - [LP Token](#lp-token)
      - [Examples](#examples-1)
    - [Contract](#contract)
      - [Examples](#examples-2)
      - [Notes](#notes-1)
    - [Ibc Channel](#ibc-channel)
      - [Examples](#examples-3)
    - [Asset pairs](#asset-pairs)
      - [Examples](#examples-4)
    - [Pool Metadata](#pool-metadata)
      - [Examples](#examples-5)
  - [Storage maps](#storage-maps)


## Delimiter Definitions

### Delimiter targets
  - asset - native/cw20/factory coins mappings used inside abstract ans, example: `juno>juno`(mapping for ujunox inside juno chain), `juno>wynd`(mapping for contract address of cw20 wynd coin inside juno chain)
  - type - protocol type, examples: `juno/croncat/factory`(chain protocol, app protocol, contract name), `juno:staking/wynd/juno>juno,juno>wynd` (protocol, contract(dapp protocol, additional info which is assets for this staking target))
  - attribute - abstract namespace attribute, abstract app name attribute or pool type, examples: `abstract:proxy`, (namespace, app), `junoswap/uusd,uust:stable` (chain protocol, assets, pool type)

### Delimiters
  - `,` - delimiter between assets, example: asset1,asset2
  - `/` - delimiter between types, example: contract_type/asset1,asset2
  - `:` - delimiter between attributes, example: abstract_namespace:contract
  - `>` - delimiter between chain(s) and asset, example: chain1>chain2>asset

### Delimiter priorities
Priority of delimiters, highest priority will overwrite delimiter from both sides
  1. `:`
  2. `/`
  3. `,` and  `>`

## Naming rules

    * All names must be lowercase
    * Assets must be prexfixed with the chain name, seperated by '>'

## Items

### Asset
- Key: `$CHAIN_NAME>$ASSET_NAME`
- Value: `{ "<native\|cw20\|cw1155>": "$ASSET_ADDRESS" }`
#### Examples
- Key:

      ```json
      "terra2>luna"
      ```

      ```json
      "terra2>stb"
      ```

- Value:

      ```json
      { "native": "uluna" }
      ```

      ```json
      { "cw20": "terra...ehcu2" }
      ```

#### Notes
All assets must be alphabetized

### LP Token
- Key: `$PROTOCOL/$ASSET_A,$ASSET_B[,$ASSET_X+]`
- Value: `{ "<native\|cw20\|cw1155>": "$ASSET_ADDRESS" }`

#### Examples
- Key: 

      ```json
      "junoswap/juno>dao,juno>juno"
      ```

      ```json
      "kujira/kujira>kuji,kujira>demo"
      ```

- Value:

      ```json
      { "cw20": "juno...vfpn2" }
      ```

      ```json
      { "native": "factory/kujira...g5cq/ulp"}
      ```

### Contract
- Key: `{ "protocol": "$PROTOCOL", "contract": "$ADDITIONAL_INFO" }`
- Value: `$CONTRACT_ADDRESS`

#### Examples
- Key:

      ```json
      { "protocol": "astroport", "contract": "staking/astroport/terra2>astro,terra2>luna" }
      ```
 
      ```json
      { "protocol": "croncat", "contract": "factory" }
      ```

- Value: 
    
      ```json
      "terra...aoeu"
      ```

#### Notes
- Staking contracts will belong here as well

### Ibc Channel
- Key: `{ "connected_chain": "$HOST_CHAIN", "protocol": "$PROTOCOL" }`
- Value: `$CHANNEL_ID`

#### Examples
- Key:

      ```json
      { "connected_chain": "osmosis", "protocol": "ics20" }
      ```

- Value:
 
      ```json
      "channel-13"
      ```

### Asset pairs
- Key: `[$ASSET_A, $ASSET_B, $DEX_NAME]`
- Value: `[{ "unique_id": "$ABSTRACT_POOL_ID", "pool_address": { "<contract\|id|separate_addresses>": $POOL_ADDRESS_ID_OR_SEPARATE_ADDRESSES } }]`

#### Examples
- Key: 

      ```json
      ["terra2>astro", "terra2>luna", "astroport"]
      ```

      ```json
      ["kujira>kuji", "kujira>demo", "kujira"]
      ```

      ```json
      ["axelar>usdc", "osmosis>osmo", "osmosis"]
      ```

- Value:
    
      ```json
      [{ "unique_id": "10", "pool_address": { "contract": "terrax...aoeu" } }]
      ```

      ```json
      [{ "unique_id": "11", "pool_address": { "separate_addresses": { "swap": "kujira...4jjh", "liquidity": "kuji...g5cq"} } }]
      ```

      ```json
      [{ "unique_id": "11", "pool_address": { "id": 678 } }]
      ```

### Pool Metadata
- Key: `$ABSTRACT_POOL_ID`
- Value: `{ "dex": "$DEX", "pool_type": "<constant_product\|stable\|weighted\|liquidity_bootstrap>", "assets": ["$ASSET_A", $ASSET_B"] }`
 
#### Examples
- Key:

      ```json
      10
      ```

- Value:

      ```json
      { "dex": "astroport", "pool_type": "weighted", "assets": ["terra2>astro", "terra2>luna"] }
      ```

## Storage maps
Those maps can be found in [ans state](../packages/abstract-core/src/native/ans_host.rs)
  - [Asset](#asset) - `ASSET_ADDRESSES`
  - [LP Token](#lp-token) - `ASSET_ADDRESSES`
  - [Contract](#contract) - `CONTRACT_ADDRESSES`
  - [Ibc Channel](#ibc-channel) - `CHANNELS`
  - [Asset pairs](#asset-pairs) - `ASSET_PAIRINGS`
  - [Pool Metadata](#pool-metadata) - `POOL_METADATA`
