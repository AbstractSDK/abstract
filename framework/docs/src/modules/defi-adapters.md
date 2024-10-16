# DeFi Adapters

Developers teams shouldn't waste engineering time and capital on integrations. Most DeFi applications rely on the same common protocols, interfaces for which are often re-implemented by every new developer team.

Abstract's DeFi [adapters](../3_framework/6_module_types.md#adapters) allow developers to write applications _agnostic_ of the underlying DeFi primitive so that they're portable to any new protocol, regardless of their APIs.

Adapters offered by Abstract currently include:

- [Dex](./dex.md)
- [Lending Market](./lending-market.md)
- [CW Staking](./cw-staking.md)

## External Use

Our DeFi adapters are open-source and can be used by any developer team, regardless of whether they're using Abstract's framework. As described, our adapters facilitate interactions with a host of DeFi primitives by handling API call construction.

To enable the "external" use of this feature we expose a `GenerateMessages` query that returns the messages required to interact with a given DeFi primitive. This query can be used by any developer team to construct the messages required to interact with a DeFi primitive.

The query is defined as:

```rust
GenerateMessages {
    /// Execute message to generate messages for
    message: <<AdapterExecuteMsg>>,
    /// Sender Addr generate messages for
    addr_as_sender: String,
}
```

Where `AdapterExecuteMsg` is the Adapter's exposed API. The query returns a list of messages (`CosmosMsg`) that should be executed by the `addr_as_sender` to interact with the DeFi primitive.
