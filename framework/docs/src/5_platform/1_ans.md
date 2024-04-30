# Abstract Name Service

The Abstract Name Service (or ANS in short) is an on-chain data store of the most important address space related data of the blockchain it is deployed on. It allows for chain-agnostic action execution and dynamic address resolution. These features enable both users and developers to engage with the blockchain in a more intuitive manner.

## ANS Architecture

The ANS is a smart contract that stores the following data:

- **Assets**: The most relevant assets on the local blockchain.
- **Contracts**: Contracts related to certain protocols or applications that could be dynamically resolved. This could
  be used to store the address for an asset-pair for a dex. For example, "osmosis/juno,osmo" could be resolved to the
  address of
  the osmosis pool that allows you to swap osmo for juno.
- **Channels**: IBC channel data to map a protocol + destination chain to a channel id. This allows for dynamic IBC
  transfers without having to know the channel id beforehand.

The ANS contract state layout is defined here. It consists of key-value mappings for the different entries.

```rust,no_run
{{#include ../../../packages/abstract-std/src/native/ans_host.rs:ans_state}}
```

```admonish info
You can find the full source code for the ANS contract <a href="https://github.com/AbstractSDK/abstract/tree/main/framework/contracts/native/ans-host" target="_blank">here</a>.
```

## Resolving Entries

The information provided by the ANS is great to have. However, directly calling CosmWasm smart queries on the ANS contract can
make your code messy and significantly raise gas usage. For this reason, we offer three methods to efficiently and
dependably execute low-gas queries on the ANS contract.

> Resolving your asset/contract name to its matching value is much like resolving a domain name like `abstract.money` to
> its IP address (`172.67.163.181`).

There are three ways to resolve your entry into its matching value.

### `AbstractNameService` Trait (Recommended)

Both App and Adapter objects implement the `AbstractNameService` trait which allows you to resolve entries.

```rust,ignore
let juno_name = AssetEntry::new("juno");
let juno_asset_info = my_app.name_service(deps).query(&juno_name)?;
```

### `Resolve` Trait

Entries that are resolvable by the Abstract Name Service implement the `Resolve` trait which gives them the ability to
be resolved by ANS explicitly.

```rust,ignore
let ans_host = my_app.ans_host(deps)?;
let juno_name = AssetEntry::new("juno");
let juno_asset_info = juno_name.resolve(&deps.querier, &ans_host)?;
```

### `AnsHost Object`

You can also load or create an `AnsHost` struct. This struct is a simple wrapper around an Addr and implements methods
that perform raw queries on the wrapped address.

```rust,ignore
let ans_host = AnsHost {address: "juno1...."};
let juno_name = AssetEntry::new("juno");
let juno_asset_info = ans_host.query_asset(deps, &juno_name)?;
```
