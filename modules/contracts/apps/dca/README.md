# DCA App Module

## Description

The DCA (Dollar Cost Averaging) app module is designed to give you an automated way to exchange an asset over time. By specifying a source asset and a target asset, you can configure the application to periodically convert a specified amount of the source asset into the target asset, effectively implementing a Dollar Cost Averaging strategy, sometimes also referred to as a "stream-swap".

## Why use the DCA App?

The advantage of this strategy is multifold. First, it allows you to move into an asset over time, allowing you to build a steady position without the short-term volatility. Secondly, big swaps can result in slippage, which can be avoided by splitting the swap into smaller chunks and executing them over time.

## Features

This module interacts with [croncat module](https://github.com/AbstractSDK/abstract/tree/main/modules/contracts/apps/croncat) to automated schedule and with [dex adapter](https://github.com/AbstractSDK/abstract/tree/main/modules/contracts/adapters/dex) for swaps!

- **Create DCA**: Sets up a new DCA strategy. This will create a new cron job that will periodically execute a swap. For each swap some gas will be used to pay for the transaction. If the `task_refill_threshold` is set, a gas-refill will automatically be attempted by the module. Ensure that the account has sufficient gas-tokens to pay for the gas-refill.
- **Cancel DCA**: End a DCA strategy. Any funds in the croncat job will be returned.
- **Update DCA**: Configure a DCA strategy by changing its assets, frequency or exchange.

## Installation

To use the DCA in your Rust project, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
dca-app = { git = "https://github.com/AbstractSDK/abstract.git", tag="<latest-tag>", default-features = false }
```

## Documentation

- **App Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/framework/module_types.html#apps).

## Contributing

If you have suggestions, improvements or want to contribute to the project, we welcome your input on [GitHub](https://github.com/AbstractSDK/abstract).

## Community
Check out the following places for support, discussions & feedback:

- Join our [Discord server](https://discord.com/invite/uch3Tq3aym)
