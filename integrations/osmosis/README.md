> :information_desk_person: If you want to use latest update from osmosis' main branch, checkout `autobuild-main` branch.

# osmosis-rust

Rust libraries for Osmosis. The following table shows every published crates maintained in this repository:

| Crate                                             | Description                                                                                                                                                            | Crates.io                                                                                                                                 | Docs                                                                                        |
| ------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------- |
| [osmosis-std](packages/osmosis-std)               | Osmosis's proto-generated types and helpers for interacting with the appchain. Compatible with CosmWasm contract.                                                      | [![osmosis-std on crates.io](https://img.shields.io/crates/v/osmosis-std.svg)](https://crates.io/crates/osmosis-std)                      | [![Docs](https://docs.rs/osmosis-std/badge.svg)](https://docs.rs/osmosis-std)               |
| [osmosis-std-derive](packages/osmosis-std-derive) | Procedural macro for augmenting proto-generated types to create better developer ergonomics. Internally used by `osmosis-std`                                          | [![osmosis-std-derive on crates.io](https://img.shields.io/crates/v/osmosis-std-derive.svg)](https://crates.io/crates/osmosis-std-derive) | [![Docs](https://docs.rs/osmosis-std-derive/badge.svg)](https://docs.rs/osmosis-std-derive) |
| [osmosis-testing]()(🚩DEPRECATED IN FAVOR OF [`osmosis-test-tube`](https://github.com/osmosis-labs/test-tube/tree/main/packages/osmosis-test-tube))       | CosmWasm x Osmosis integration testing library that, unlike `cw-multi-test`, it allows you to test your cosmwasm contract against real chain's logic instead of mocks.  | [![osmosis-testing on crates.io](https://img.shields.io/crates/v/osmosis-testing.svg)](https://crates.io/crates/osmosis-testing)          | [![Docs](https://docs.rs/osmosis-testing/badge.svg)](https://docs.rs/osmosis-testing)       |


---

This repo also contains [`proto-build`](./packages/proto-build) package which is used for autogenrating rust types from proto.
