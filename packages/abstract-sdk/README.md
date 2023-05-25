# Abstract-SDK

<!-- [![](https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github)](https://github.com/AbstractSDK/contracts)   -->

[![](https://docs.rs/abstract-sdk/badge.svg)](https://docs.rs/abstract-sdk) [![](https://img.shields.io/crates/v/abstract-sdk)](https://crates.io/crates/abstract-sdk)

This crate provides a set of modular APIs for developers to use in their [CosmWasm](https://cosmwasm.com/)
smart-contracts.

## Getting started

To get started with the Abstract SDK you first need to understand the basic features that we provide and how you can use
those features to create composable smart-contract APIs.

### Features

Abstract features are traits that can be implemented on a struct. Depending on the use-case that struct can represent a
smart-contract or it can be a simple struct that just implements a single feature. Each feature unlocks a function on
the object
which allows you to retrieve some information. By composing these features it is possible to write advanced APIs
that are automatically implemented on objects that support its required features.

### APIs

The Abstract APIs are objects that can only be retrieved if a contract or feature-object implements the required
features/api traits. If the trait constraints for the Adapter is met it is automatically implemented on the object and
allows you to retrieve the Adapter object.

#### Example

The [`Bank`](https://docs.rs/abstract-sdk/latest/abstract_sdk/apis/bank) Adapter allows developers to transfer assets from and to the Account through their module object. We now want to use this Adapter to create a `Splitter` API that splits the transfer of some amount of funds between a set of receivers.

```rust,no_run
use abstract_sdk::{TransferInterface,AbstractSdkResult};
use abstract_core::objects::AnsAsset;
use cosmwasm_std::{Addr, CosmosMsg, Deps, StdResult, Uint128};

// Trait to retrieve the Splitter object
// Depends on the ability to transfer funds
pub trait SplitterInterface: TransferInterface {
    fn splitter<'a>(&'a self, deps: Deps<'a>) -> Splitter<Self> {
        Splitter { base: self, deps }
    }
}

// Implement for every object that can transfer funds
impl<T> SplitterInterface for T where T: TransferInterface {}

#[derive(Clone)]
pub struct Splitter<'a, T: SplitterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: SplitterInterface> Splitter<'a, T> {
    /// Split an asset to multiple users
    pub fn split(&self, asset: AnsAsset, receivers: &[Addr]) -> AbstractSdkResult<Vec<CosmosMsg>> {
        // split the asset between all receivers
        let receives_each = AnsAsset {
            amount: asset
                .amount
                .multiply_ratio(Uint128::one(), Uint128::from(receivers.len() as u128)),
            ..asset
        };

        // Retrieve the bank API
        let bank = self.base.bank(self.deps);
        let transfer_msgs: AbstractSdkResult<_> = receivers
            .iter()
            .map(|receiver| {
                // Construct the transfer message
                bank.transfer(vec![&receives_each], receiver)
            })
            .collect();

        transfer_msgs
    }
}
```

The API can then be used by any contract that implements its required traits, in this case the `TransferInterface`.

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
      let send_deposit_to_vault_msg = my_contract.bank(deps).deposit_coins(message_info.funds)?;
      Ok(send_deposit_to_vault_msg)
  }
```

### Abstract Base

To use an API either construct a [`feature object`](crate::feature_objects) or use an Abstract base contract as the starting-point of your application.  
The available base contracts are:

| Kind                                                   | Migratable | Installable |
|--------------------------------------------------------|------------|-------------|
| [App](https://crates.io/crates/abstract-app)           | ✅          | ✅           |
| [Adapter](https://crates.io/crates/abstract-adapter)       | ❌          | ✅           |
| [IBC-host](https://crates.io/crates/abstract-ibc-host) | ✅          | ❌           |

Each base supports a set of endpoints that can accept custom handlers. These handlers can be added to the base using a static builder pattern.
All the available endpoints are discussed [here](crate::base::endpoints).

## Usage

Add `abstract-sdk` to your `Cargo.toml` by running:

```bash
cargo add abstract-sdk
```
