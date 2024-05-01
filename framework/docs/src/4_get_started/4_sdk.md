# SDK

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

Now that that you're familiar with construction of your module you're ready for our hot sauce. While you can traditional code in your module, using our SDK will give you a huge productivity boost. In short, we've created an **account abstraction programming toolbox** that allows you to easily control an Abstract Account's interactions, as well as **create your own APIs** that can be used by other developers to interact with your unique application. Composability galore!

## APIs

Abstract API objects are Rust structs that expose some smart contract functionality. Such an API object can only be constructed if a contract implements the traits that are required for that API. Access to an API is automatically provided if the trait constraints for the API are met by the contract.

> We've created a set of APIs that can be used to interact with the Abstract Account and have implemented their trait requirements on the module base types that we provide (`App` and `Adapter`). So for you, it's just plug and play! 🎉

Most of the APIs either return a `CosmosMsg` or an `AccountAction`.

### `CosmosMsg` Example

 The `CosmosMsg` is a message that should be added as-is to the `Response` to perform some action.

This example sends coins from the local contract (module) to the account that the application is installed on.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/bank.rs:deposit }}

            Ok(response)
```
[source](https://github.com/AbstractSDK/abstract/blob/main/framework/packages/abstract-sdk/src/apis/bank.rs#L321)

Custom `CosmosMsgs` can be added in the same way through the `app.response("<action>")` function. The `action` attribute of the function is a string that will be added to the response's attributes and will be available in the transaction result under the `wasm-abstract` event. This way you can easily figure out which actions were called in a tx!

The above example can equally be written as:

```rust
let coins: Vec<Coin> = coins(100u128, "denom");
// Create CosmosMsg
let bank_msg: CosmosMsg = CosmosMsg::Bank(BankMsg::Send {
    to_address: "<proxy_address>".to_string(),
    amount: coins,
});
// Add to Response
let response: Response = app.response("deposit").add_message(deposit_msg);

Ok(response)
```

This gives you all the flexibility you are used to when working with CosmWasm!

### `AccountAction` Example

The other kind of struct that can be returned by an Abstract API is the `AccountAction`. An `AccountAction` is a single, or collection of `CosmosMsgs` that should be executed on the App's Abstract Account.

`AccountActions` can be executed with the [`Executor`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Executor.html) API. The returned `CosmosMsg` should be added to the action's `Response`.

The following example sends coins **from the account** to another address. This action requires the account itself to execute the message and transfer the funds.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/bank.rs:transfer }}
```

[source](https://github.com/AbstractSDK/abstract/blob/main/framework/packages/abstract-sdk/src/apis/bank.rs#L277)

So through the `Executor` API you can execute messages on behalf of the Account! Also notice that you can provide multiple actions to the executor to be executed in sequence.

## How it works

As you're aware, `abstract-sdk` crate is a toolbox for developers to create composable smart contract APIs. It does this through a combination of supertraits and blanket implementations, two concepts that are native to the Rust language.

```admonish info
*Supertraits* are Rust traits that have one or multiple trait bounds while a *blanket implementation* is a Rust trait implementation that is automatically implemented for every object that meets that trait's trait bounds. The Abstract SDK uses both to achieve its modular design.

For more information about traits, supertraits and blanket implementations, check out the Rust documentation:

- <a href="https://doc.rust-lang.org/book/ch10-02-traits.html" target="_blank">Traits</a>
- <a href="https://doc.rust-lang.org/book/ch10-02-traits.html#traits-as-parameters" target="_blank">Supertraits</a>
- <a href="https://doc.rust-lang.org/book/ch10-02-traits.html#implementing-a-trait-on-a-type" target="_blank">Blanket
  Implementations</a>
```

## Features

Features are the lowest-level traits that are contained within the SDK and they don't have any (custom) trait bounds. They generally act as data accessor traits. I.e. if a struct implements a feature it means that it has some way to get the
information required by that feature.

Here's an example of such a feature:

```rust
{{ #include ../../../packages/abstract-sdk/src/base/features/abstract_name_service.rs:ans }}
```

Any structure that implements this trait has access to the `AnsHost` struct, which is a wrapper around an `Addr`. Because that structure now has the address of that contract, it can resolve ANS entries.

Now instead of letting you implement these traits yourself, we've already gone ahead and implemented them for the `App` and `Adapter` structs.

So when you're building your application, the module struct already has the features and data required to do abstract operations (😉). With this in place we can start creating more advanced functionality.

```admonish info
Other structs that implement a feature without being module bases are called *Feature Objects*.
```

## Usage

Add `abstract-sdk` to your `Cargo.toml` by running:

```bash
cargo add abstract-sdk
```

Then import the prelude in your contract. This will ensure that you have access to all the traits which should help your IDE with auto-completion.

```rust,ignore
use abstract_sdk::prelude::*;
```

## Creating your own API

The <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Bank.html" target="_blank">`Bank`</a> API allows developers to transfer assets from and to the Account. We now want to use this API to create a `Splitter` API that splits the transfer of some amount of funds between a set of receivers.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/splitter.rs:splitter }}
```

<a href="https://github.com/AbstractSDK/abstract/blob/main/framework/packages/abstract-sdk/src/apis/splitter.rs" target="_blank">source</a>

These APIs can then be used by any contract that implements its required traits, in this case the `TransferInterface`.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/splitter.rs:usage }}
```

## Appendix

### Available API Objects

The following API objects are available in the Abstract SDK:

- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Bank.html" target="_blank">`Bank`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Executor.html" target="_blank">`Executor`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Apps.html" target="_blank">`Apps`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Adapters.html" target="_blank">`Adapters`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Accountant.html" target="_blank">`Accountant`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.IbcClient.html" target="_blank">`IbcClient`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.ModuleRegistry.html" target="_blank">`ModuleRegistry`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Modules.html" target="_blank">`Modules`</a>
- <a href="https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.AccountRegistry.html" target="_blank">`AccountRegistry`</a>

Other projects have also started building APIs. Here are some examples:

- <a href="https://github.com/CronCats/abstract-croncat-app/blob/main/src/api.rs" target="_blank">`Cron Cats`</a>
- More coming soon...
