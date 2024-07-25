# Dex Adapter Module

The Dex Adapter Module provides a unified interface to interact with various decentralized exchanges (dexes) across the Cosmos ecosystem. By abstracting the differences between various dexes, it allows developers to interact with any dex using a standard interface, streamlining the development process and ensuring compatibility across various dex platforms.

## Features

- **Swap**: Exchange one asset for another.
- **Provide Liquidity**: Add assets to a liquidity pool.
- **Withdraw Liquidity**: Remove assets from a liquidity pool.
- **Simulate Swap**: Predict the outcome of a swap without executing it, useful for previewing potential trades.
- **Provide Liquidity Symmetric**: Add an equal value of two assets to a liquidity pool.
- **Custom Swap**: Execute a swap with custom parameters, allowing for more advanced trading strategies.

```admonish info
Note that each one of these actions supports both ANS and raw variants, meaning that you can use both human-readable and explicit asset denominations.
```

## Supported Dexes

The following Dexes are currently supported:

- Osmosis (Osmosis)
- Astroport (Neutron, Terra, Injective, Sei)
- Kujira (Kujira)
- Astrovault (Archway)
- Wyndex (Juno)

If you would like to request support for an additional Dex, please create a GitHub issue or reach out to us on Discord.

## Installation

To use the Dex Adapter Module in your Rust project, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
abstract-dex-adapter = { git = "https://github.com/AbstractSDK/abstract.git", tag="v0.18.0", default-features = false }
```

## Usage with the Abstract SDK

To interact with a dex, you first need to retrieve the dex using the Dex Adapter. Here's a basic example in Rust:

```rust
// Retrieve the dex
use abstract_dex_adapter::api::DexInterface;
...

let dex_name = "osmosis".to_string();
let offer_asset = OfferAsset::new("juno", 1000u128);
let ask_asset = AssetEntry::new("uusd");
let max_spread = Some(Decimal::percent(1));
let belief_price = Some(Decimal::percent(2));

let dex = app.dex(deps.as_ref(), dex_name);

let swap_msg = dex.swap(offer_asset, ask_asset, max_spread, belief_price);
```

## Why Use the Dex Adapter?

### Simplified Development
By using the Dex Adapter, developers can bypass the intricacies of each individual dex. This means less time spent on understanding and integrating with each dex's unique API, and more time focusing on building core functionalities.

### Flexibility
The Dex Adapter ensures that your application remains flexible. If a new dex emerges or if there are changes to an existing one, your application can easily adapt without undergoing major overhauls.

### Use Cases
- **Rapid Prototyping**: Quickly build and test applications on top of various dexes without the need for multiple integrations.
- **Cross-Dex Applications**: Build applications that leverage multiple dexes simultaneously, offering users more options and better rates.
- **Future-Proofing**: Ensure your application remains compatible with future dexes that emerge in the Cosmos ecosystem.

## Documentation

- **Dex Interface**: For a detailed look at the dex interface, refer to the [Rust trait interface](https://github.com/AbstractSDK/abstract/blob/bcf26f2f446478fd2825de5b187321dc9a626341/modules/contracts/adapters/dex/src/api.rs#L43).
- **Adapters Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/6_module_types.html#adapters).

## Contributing

If you have suggestions, improvements, new dexes, or want to contribute to the project, we welcome your input on GitHub.
