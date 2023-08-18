# Naming Convention Reference

## Delimiter Definitions

### Delimiter targets
  - asset - native/cw20/factory coins mappings used inside abstract ans, example: `juno>juno`(mapping for ujunox inside juno chain), `juno>wynd`(mapping for contract address of cw20 wynd coin inside juno chain)
  - type - protocol type, examples: `juno/croncat/factory`(chain protocol, app protocol, contract name), `juno:staking/wynd/juno>juno,juno>wynd` (protocol, contract(dapp protocol, additional info which is assets for this staking target))
  - attribute - abstract namespace attribute, abstract app name attribute or pool type, examples: `abstract:proxy`, (namespace, app), `junoswap/uusd,uust:stable` (chain protocol, assets, pool type)

### Delimiters: 
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
    * assets must be prexfixed with the chain name, seperated by '>'
    * TODO: more

## Items

### Asset
- Key: `$CHAIN_NAME>$ASSET_NAME`
- Value: `{ "<native\|cw20\|cw1155>": "$ASSET_ADDRESS" }`
#### Examples
- Key: `terra2>luna`, `terra2>stb`
- Value: `{ "native": "uluna" }`, `{ "cw20": "terra...ehcu2" }`

### LP Token
- Key: `$PROTOCOL/$ASSET_A,$ASSET_B[,$ASSET_X+]`
- Value: `{ "<native\|cw20\|cw1155>": "$ASSET_ADDRESS" }`

#### Examples
- Key: `junoswap/juno>dao,juno>juno`, `kujira/kujira>kuji,kujira>demo`
- Value: `{ "cw20": "juno...vfpn2" }`, `{ "native": "factory/kujira...g5cq/ulp"}`

#### Notes
TODO: why this note was here and not in assets? 
All assets must be alphabetized

### Contract
- Key: `{ "protocol": "$PROTOCOL", "contract": "$ADDITIONAL_INFO" }`
- Value: `$CONTRACT_ADDRESS`

#### Examples
- Key: `{ "protocol": "astroport", "contract": "staking/astroport/terra2>astro,terra2>luna" }`, `{ "protocol": "croncat", "contract": "factory" }`
- Value: `terra...aoeu`

#### Notes
- Staking contracts will belong here as well

### Ibc Channel
- Key: `{ "connected_chain": "$HOST_CHAIN", "protocol": "$PROTOCOL" }`
- Value: `$CHANNEL_ID`

#### Examples
- Key: `{ "connected_chain": "osmosis", "protocol": "ics20" }`
- Value: `channel-13`

### Asset pairs
- Key: `[$ASSET_A, $ASSET_B, $DEX_NAME]`
- Value: `[{ "unique_id": "$ABSTRACT_POOL_ID", "pool_address": { "<contract\|id|separate_addresses>": $POOL_ADDRESS_ID_OR_SEPARATE_ADDRESSES } }]`

#### Examples
- Key: `["terra2>astro", "terra2>luna", "astroport"]`, `["kujira>kuji", "kujira>demo", "kujira"]`, `["axelar>usdc", "osmosis>osmo", "osmosis"]`
- Value: `[{ "unique_id": "10", "pool_address": { "contract": "terrax...aoeu" } }]`, `[{ "unique_id": "11", "pool_address": { "separate_addresses": { "swap": "kujira...4jjh", "liquidity": "kuji...g5cq"} } }]`, `[{ "unique_id": "11", "pool_address": { "id": 678 } }]`

### Pool Metadata
- Key: `$ABSTRACT_POOL_ID`
- Value: `{ "dex": "$DEX", "pool_type": "<constant_product\|stable\|weighted\|liquidity_bootstrap>", "assets": ["$ASSET_A", $ASSET_B"] }`
 
#### Examples
- Key: `10`
- Value: `{ "dex": "astroport", "pool_type": "weighted", "assets": ["terra2>astro", "terra2>luna"] }`

## Storage keys for raw querying

Below is the human-readable version of your markdown table:

| Type | Storage | Key | Ex Key | Value | Ex Value | Notes |
| --- | --- | --- | --- | --- | --- | --- |
| Asset | `ASSET_ADDRESSES` | `$chain>$asset_name` | `terra2>luna` | `{ "<native\|cw20\|cw1155>": "$ASSET_ADDRESS" }` | `{ "native": "uluna }` |  |
| LP Token | `ASSET_ADDRESSES` | `$PROTOCOL/$ASSET_A,$ASSET_B[,$ASSET_X+]` | `junoswap/juno>dao,juno>juno` | `{ "<cw20\|cw1155>": "$ASSET_ADDRESS" }` | `{ "cw20": "juno...vfpn2" }` | All assets must be alphabetized |
| Contract | `CONTRACT_ADDRESSES` | `{ "protocol": "$PROTOCOL", "contract": "$TYPE/$PROTOCOL/$ADDITIONAL_INFO", }` | `{ "protocol": "astroport", "contract": "staking/astroport/terra2>astro,terra2>luna" }` | `$CONTRACT_ADDRESS` | `terra...aoeu` |  |
| Staking Contract | `CONTRACT_ADDRESSES` | `{ "protocol": "$PROTOCOL", "contract": "staking/$PROTOCOL/$ASSET_A,$ASSET_B[,$ASSET_X+]" }` | `{ "protocol": "astroport", "contract": "staking/astroport/terra2>astro,terra2>luna" }` | `$CONTRACT_ADDRESS` | `terra...aoeu` |  |
| Ibc Channel | `CHANNELS` | `{ "connected_chain": "$HOST_CHAIN", "protocol": "$PROTOCOL" }` | `{ "connected_chain": "osmosis", "protocol": "ics20" }` | `$CHANNEL_ID` | `channel-13` |  |
| Asset pairs | `ASSET_PAIRINGS` | `[$ASSET_A, $ASSET_B, $DEX_NAME]` | `["terra2>astro", "terra2>luna", "astroport"]` | `[{ "unique_id": "$ABSTRACT_POOL_ID", "pool_address": { "<contract\|id>": "$POOL_ADDRESS_OR_ID" } }]` | `[{ "unique_id": "10", "pool_address": { "contract": "terrax...aoeu" } }]` |  |
| Pool Metadata | `POOL_METADATA` | `$ABSTRACT_POOL_ID` | 10 | `{ "dex": "$DEX", "pool_type": "<constant_product\|stable\|weighted\|liquidity_bootstrap>", "assets": ["$ASSET_A", $ASSET_B"] }` | `{ "dex": "astroport", "pool_type": "weighted", "assets": ["terra2>astro", "terra2>luna"] }` |  |
