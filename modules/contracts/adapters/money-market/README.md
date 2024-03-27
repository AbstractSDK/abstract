# MoneyMarket Adapter Module

The MoneyMarket Adapter Module provides a unified interface to interact with various lending and borrowing markets (moneymarkets) across the Cosmos ecosystem. By abstracting the differences between various moneymarkets, it allows developers to interact with any moneymarket using a standard interface, streamlining the development process and ensuring compatibility across various moneymarket platforms.

## Features

- **Swap**: Exchange one asset for another.
- **Provide Liquidity**: Add assets to a liquidity pool.
- **Withdraw Liquidity**: Remove assets from a liquidity pool.
- **Simulate Swap**: Predict the outcome of a swap without executing it, useful for previewing potential trades.
- **Provide Liquidity Symmetric**: Add an equal value of two assets to a liquidity pool.
- **Custom Swap**: Execute a swap with custom parameters, allowing for more advanced trading strategies.

## Installation

To use the MoneyMarket Adapter Module in your Rust project, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
abstract-money-market-adapter = { git = "https://github.com/AbstractSDK/abstract.git", tag="v0.21.1", default-features = false }
```

## Usage with the Abstract SDK

To interact with a moneymarket, you first need to retrieve the moneymarket using the Moneymarket Api. Here's a basic example in Rust:

```rust
// Retrieve the money_market
use abstract_money_market_adapter::api::MoneyMarketInterface;
...

let money_market_name = "mars".to_string();
let deposit_asset = OfferAsset::new("juno", 1000u128);

let money_market = app.money_market(deps.as_ref(), money_market_name);
let deposit_msg = money_market.deposit(deposit_asset);
```

## Limitation

The Money Market adapter provides easy ways of interacting with Money Markets. However, some errors can appear without the adapter catching them:

- The money market can have deposit limits enabled which may be crossed when using this adapter.
- The money market may not have liquidity available to borrow funds.
- The money market may not have liquidity available to withdraw deposited funds from
- The user may not be able to withdraw collateral because they are borrowing too much funds.
  
All those errors and more have to be handled directly by the developers integrating this adapter.

## Why Use the MoneyMarket Adapter?

### Simplified Development

By using the Adapter, developers can bypass the intricacies of each individual platform. This means less time spent on understanding and integrating with each moneymarket's unique API, and more time focusing on building core functionalities.

### Flexibility

The MoneyMarket Adapter ensures that your application remains flexible. If a new moneymarket emerges or if there are changes to an existing one, your application can easily adapt without undergoing major overhauls.

### Use Cases

- **Rapid Prototyping**: Quickly build and test applications on top of various moneymarkets without the need for multiple integrations.
- **Cross-Dex Applications**: Build applications that leverage multiple moneymarkets simultaneously, offering users more options and better rates.
- **Future-Proofing**: Ensure your application remains compatible with future moneymarkets that emerge in the Cosmos ecosystem.

## Documentation

- **Moneymarket Interface**: For a detailed look at the moneymarket interface, refer to the [Rust trait interface](https://github.com/AbstractSDK/abstract/tree/main/modules/contracts/adapters/moneymarket/src/api.rs#L43). #TODO, fix this will be broken

- **Adapters Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/7_module_types.html#adapters).

## Contributing

If you have suggestions, improvements, new moneymarkets, or want to contribute to the project, we welcome your input on GitHub.
