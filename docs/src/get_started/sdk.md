# SDK

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

Now that you've got your module set up you're probably wondering how you can actually start programming. While you can create any regular smart-contract in a module, it's not a use-case where our platform excels in. Instead we've created an **account abstraction programming toolbox** that allows you to easily control the Abstract Account, as well as **create your own APIs** that can be used by other developers to interact with your unique application. Composability galore!

## How it works

The `abstract-sdk` crate is a toolbox for developers to create composable smart-contract APIs. It allows you to use composed functionality with a few keystrokes through it's combination of supertraits and blanket implementations. Supertraits are Rust traits that have one or multiple trait bounds while a blanket implementation is a Rust implementation that is automatically implemented for every object that meets the trait bounds. The Abstract SDK makes use of both to achieve its modular design.

### Features

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

### APIs

TODO: Update with SDK execution changes

The Abstract API objects are structs that expose some smart-contract functionality and that can only be retrieved if a contract or feature-object implements the required features/api traits. If the trait constraints for the API is met it is automatically implemented on the object and hence allows you to retrieve the API object.

#### Example

The [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/apis/bank) API allows developers to transfer assets from and to the Account through their module object. We now want to use this API to create a `Splitter` API that splits the transfer of some amount of funds between a set of receivers.

```rust,ignore
{{#include ../../../packages/abstract-sdk/src/apis/splitter.rs }}
```

These APIs can then be used by any contract that implements its required traits, in this case the `TransferInterface`.

```rust,no_run
# use abstract_sdk::features::{AccountIdentification, AbstractNameService, ModuleIdentification};
# use cosmwasm_std::{StdResult, Deps, MessageInfo, CosmosMsg, Addr};
# use abstract_sdk::feature_objects::AnsHost;
# use abstract_sdk::AbstractSdkResult;
# pub struct MyContract {
#     
# }
# impl AccountIdentification for MyContract {
#     fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
#         Ok(Addr::unchecked("just_an_example"))
#     }
# }
# impl ModuleIdentification for MyContract {
#     fn module_id(&self) -> &'static str { "my_contract" }
# }
# impl AbstractNameService for MyContract {
#     fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
#         Ok(AnsHost{address: Addr::unchecked("just_an_example")})
#     }
# }
use abstract_sdk::TransferInterface;

fn forward_deposit(deps: Deps, my_contract: MyContract, message_info: MessageInfo) -> AbstractSdkResult<CosmosMsg> {
    // Construct the API object
    let bank: Bank = my_contract.bank(deps);
    // Use the API to deposit the coins
    let send_deposit_to_vault_msg = bank.deposit_coins(message_info.funds)?;
    Ok(send_deposit_to_vault_msg)
}
```

### Available API Objects

The following API objects are available in the Abstract SDK:

- [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Bank.html)
- [`Executor`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Executor.html)
- [`App`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.App.html)
- [`Adapter`](https://docs.rs/abstract-sdk/latest/abstract_sdk/struct.Adapter.html)


## Usage

Add `abstract-sdk` to your `Cargo.toml` by running:

```bash
cargo add abstract-sdk
```

Then import the prelude in your contract. This will ensure that you have access to all the traits which should help your IDE with auto-completion.

```rust,ignore
use abstract_sdk::prelude::*;
```
