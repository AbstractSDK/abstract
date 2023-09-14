# CosmWasm Staking

An Abstract Adapter module that handles staking and unbonding interactions with staking providers. 

## Naming Convention

In order to easily identify and relate contracts to on-chain addresses we follow the following conventions:

*Staking AddressEntry*: a `ContractEntry` that is formatted as {provider}:stake/{staking_asset_entry}
*Staking AssetEntry*: a `AssetEntry` of the token that is stakeable


# CosmWasm Staking Adapter Module

The CosmWasm Staking Adapter Module provides a unified interface to interact with various protocols offering staking services to token holders. This can go from LP-staking to governance token locking. By abstracting the differences between various staking protocols, it allows developers to interact with any dex using a standard interface, streamlining the development process and ensuring compatibility across various staking platforms.

## Features

- **Stake**: Deposit (and lock) assets inside a staking protocol
- **Unstake**: Remove (and unlock) assets from staking protocol. In protocols with a locking period after unstaking, this simply triggers the unbonding process
- **Claim Rewards**: Claims the rewards associated with locking your assets. In protocols with a locking period on rewad claim, this simply triggers the unbonding process
- **Claim**: Claim matured unbonding claims (from unstake of Claim rewards)

## Installation

To use the Dex Adapter Module in your Rust project, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
abstract-cw-staking = { git = "https://github.com/AbstractSDK/abstract.git", tag="v0.18.0", default-features = false }
```

## Usage with the Abstract SDK

```rust
// Retrieve the staking interface
use abstract_cw_staking_adapter::api::CwStakingAdapter;
...

let staking_name = "osmosis".to_string();
let offer_asset = OfferAsset::new("juno", 1000u128);
let ask_asset = AssetEntry::new("uusd");
let max_spread = Some(Decimal::percent(1));
let belief_price = Some(Decimal::percent(2));

let dex = app.dex(deps.as_ref(), dex_name);

let swap_msg = dex.swap(offer_asset, ask_asset, max_spread, belief_price);
```

## Why Use the Cw Staking Adapter?

### Simplified Development
By using the Cw Staking Adapter, developers can bypass the intricacies of each individual Staking Provider. This means less time spent on understanding and integrating with each staking provider's unique API, and more time focusing on building core functionalities.

### Flexibility
The DexCw Staking Adapter ensures that your application remains flexible. If a new Staking provider or use-case emerges or if there are changes to an existing one, your application can easily adapt without undergoing major overhauls.

### Use Cases
- **Rapid Prototyping**: Quickly build and test applications on top of various staking providers without the need for multiple integrations.
- **Cross-Dex Applications**: Build applications that leverage multiple Staking providers simultaneously, offering users more options and better rates.
- **Future-Proofing**: Ensure your application remains compatible with future staking solutions that emerge in the Cosmos ecosystem.

## Documentation

- **Dex Interface**: For a detailed look at the cw-staking interface, refer to the [Rust trait interface](https://github.com/AbstractSDK/abstract/blob/bcf26f2f446478fd2825de5b187321dc9a626341/modules/contracts/adapters/cw-staking/src/api.rs#L43).
- **Adapters Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/7_module_types.html#adapters).

## Contributing

If you have suggestions, improvements, new dexes, or want to contribute to the project, we welcome your input on GitHub.