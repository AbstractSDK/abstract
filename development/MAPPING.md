# Naming Convention Reference

Naming rules:

    * All names must be lowercase
    * assets must be prexfixed with the chain name, seperated by '>'
    * contract addresses 
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
