# Oracle Adapter Module

The Oracle Adapter Module provides a unified interface to interact with various oracle providers across the Cosmos ecosystem. By abstracting the differences between various oracles, it allows developers to interact with any oracle source using a standard interface, streamlining the development process and ensuring compatibility across various oracle platforms.

## Features

The Oracle Adapter allows developers to query a price associated with a price source key. The module provides the following simple interface:

```rust
pub enum OracleQueryMsg{
    Price{
        oracle: String,
        price_source_key: String,
        // Only successful if price is not too old
        max_age: u64
    }
}
// And returns
pub struct PriceResponse{
    price: cosmwasm_std::Decimal
}

```

## Supported Oracle Providers

The following Oracles are currently supported:

- Pyth (Xion, Neutron, Osmosis)

If you would like to request support for an additional oracle, please create a GitHub issue or reach out to us on Discord.

## Installation

To use the Oracle Adapter Module in your Rust project, add the following dependency to your `Cargo.toml`:

```toml
[dependencies]
abstract-oracle-adapter = { git = "https://github.com/AbstractSDK/abstract.git", tag="<latest-tag>", default-features = false }
```

where at the time of writing, `latest-tag=v0.25.0`

## Usage with the Abstract SDK

To interact with an oracle, inside an Abstract module, you first need to retrieve the oracle using the Oracle Adapter. Here's a basic example in Rust:

```rust
use abstract_oracle_adapter::api::OracleInterface;
...

let oracle_name = "pyth".to_string();
let price_source_key = "e62df6c8b4a85fe1a67db44dc12de5db330f7ac66b72dc658afedf0f4a415b43";

// Retrieve the oracle
let oracle = app.oracle(deps.as_ref(), oracle_name);
// My custom contract needs prices that are not older than 1 minute
let max_age = 60u64;

let price = oracle.price(price_source_key, max_age);
```

## Why Use the Oracle Adapter?

### Simplified Development

By using the Oracle Adapter, developers can bypass the intricacies of each individual oracle. This means less time spent on understanding and integrating with each oracles's unique API, and more time focusing on building core functionalities.

### Flexibility

The Oracle Adapter ensures that your application remains flexible. If a new oracle emerges or if there are changes to an existing one, your application can easily adapt without undergoing major overhauls.

### Use Cases

- **Rapid Prototyping**: Quickly build and test applications on top of various oracles without the need for multiple integrations.
- **Safer Applications**: Build applications that leverage multiple oracles simultaneously, offering users more stability and safety for their asset value.
- **Future-Proofing**: Ensure your application remains compatible with future oracles that emerge in the Cosmos ecosystem.

## Tests

Tests for this implementation can be found in different locations:

- Either inside this crate for tests that can be run inside Mock or OsmosisTestTube
- In the `/interchain/modules-clone-testing` crate for integrations that require CloneTesting (e.g. Pyth)?

## Documentation

- **Oracle Interface**: For a detailed look at the oracle interface, refer to the [Rust trait interface](https://github.com/AbstractSDK/abstract/blob/bcf26f2f446478fd2825de5b187321dc9a626341/modules/contracts/adapters/oracle/src/api.rs#L38).
- **Adapters Documentation**: Comprehensive information about adapters can be found in the [official documentation](https://docs.abstract.money/3_framework/6_module_types.html#adapters).

## Contributing

If you have suggestions, improvements, new oracles, or want to contribute to the project, we welcome your input on GitHub.
