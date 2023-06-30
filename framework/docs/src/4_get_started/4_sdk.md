# SDK

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

Now that you've got your module set up you're ready for our hot sauce. While you can create any regular smart-contract in a module, it's where our software shines. Instead we've created an **account abstraction programming toolbox** that allows you to easily control an Abstract Account's interactions, as well as **create your own APIs** that can be used by other developers to interact with your unique application. Composability galore!

## How it works

The `abstract-sdk` crate is a toolbox for developers to create composable smart-contract APIs. It allows you to use composed functionality with a few keystrokes through it's combination of supertraits and blanket implementations. Supertraits are Rust traits that have one or multiple trait bounds while a blanket implementation is a Rust implementation that is automatically implemented for every object that meets the trait bounds. The Abstract SDK uses both to achieve its modular design.

## APIs

Abstract API objects are Rust structs that expose some smart-contract functionality. Such an API can only be retrieved if a contract (or feature-object) implements the required features/api traits. Access to an API is automatically provided if the trait constraints for the API are met by the contract.

Most of the APIs either return a `CosmosMsg` or an `AccountAction`. The `CosmosMsg` is a message that should be added as-is to the `Response` to perform some action.

### `CosmosMsg` Example

This example sends coins from the local contract (module) to the account that the application is installed on which does not require the account itself to execute the action.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/bank.rs:deposit }}
```

Alternatively `AccountAction` structs can also be returned by an API. An `AccountAction` is supposed to be forwarded to the Abstract Account to let the account perform action. `AccountAction`s can be executed with the [`Executor`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Executor.html) API. The returned `CosmosMsg` should be added to the action's `Response`.

### `AccountAction` Example

This example sends coins from the account to another address which requires the account itself to execute the action.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/bank.rs:transfer }}
```

## Creating your own API

The [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Bank.html) API allows developers to transfer assets from and to the Account. We now want to use this API to create a `Splitter` API that splits the transfer of some amount of funds between a set of receivers.

> The code behind this example is available [here](https://github.com/AbstractSDK/contracts/blob/main/packages/abstract-sdk/src/apis/splitter.rs).

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/splitter.rs:splitter }}
```

These APIs can then be used by any contract that implements its required traits, in this case the `TransferInterface`.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/splitter.rs:usage }}
```

## Available API Objects

The following API objects are available in the Abstract SDK:

- [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Bank.html)
- [`Executor`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Executor.html)
- [`Apps`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Apps.html)
- [`Adapters`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Adapters.html)
- [`Accountant`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Accountant.html)
- [`IbcClient`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.IbcClient.html)
- [`ModuleRegistry`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.ModuleRegistry.html)
- [`Modules`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Modules.html)
- [`AccountRegistry`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.AccountRegistry.html)

Other projects have also started building APIs. Here are some examples:

- [`Cron Cats`](https://github.com/CronCats/abstract-croncat-app/blob/main/src/api.rs)
- More coming soon...

## Features

Features are the lowest-level traits that are contained within the SDK and they don't have any trait bounds. They generally act as data accessor traits. I.e. if a struct implements a feature it means that it has some way to get the information required by that feature.

Here's an example of such a feature:

```rust
{{#include ../../../packages/abstract-sdk/src/base/features/abstract_name_service.rs:ans }}
```

Any structure that implements this traits has access to the Abstract Name Service, and thus has a way to resolve ANS entries. By composing these features it is possible to write advanced APIs that are automatically implemented on objects that support its required features.

Now instead of letting you implement these traits yourself, we've already gone ahead and implemented them for the App and Adapter structs. Here's the implementation for the App:

```rust
{{#include ../../../packages/abstract-app/src/features.rs:ans }}
```

So when you're building your application the module struct already has the features and data required to do the basic abstract operations. With this we can start creating more advanced functionality.

> Other structs that implement a feature without being module bases are called *Feature Objects*.

## Usage

Add `abstract-sdk` to your `Cargo.toml` by running:

```bash
cargo add abstract-sdk
```

Then import the prelude in your contract. This will ensure that you have access to all the traits which should help your IDE with auto-completion.

```rust,ignore
use abstract_sdk::prelude::*;
```
