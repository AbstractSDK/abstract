//! [![github]](https://github.com/Abstract-OS/contracts)&ensp;[![crates-io]](https://crates.io/crates/abstract-sdk)&ensp;[![docs-rs]](https://docs.rs/abstract-sdk)
//!
//! [github]: https://img.shields.io/badge/github-8da0cb?style=for-the-badge&labelColor=555555&logo=github
//! [crates-io]: https://img.shields.io/badge/crates.io-fc8d62?style=for-the-badge&labelColor=555555&logo=rust
//! [docs-rs]: https://img.shields.io/badge/docs.rs-66c2a5?style=for-the-badge&labelColor=555555&logo=docs.rs
//! <br>  
//! </br>
//! This crate provides a set of modular APIs for developers to use in their [CosmWasm](https://cosmwasm.com/) smart-contracts.
//!
//! # Details
//! To use an API either construct a [`feature object`](crate::feature_objects) or use an Abstract base contract as the starting-point of your application.  
//! The available base contracts are:
//! > - [Add-on](https://crates.io/crates/abstract-app) ([Template](https://github.com/Abstract-OS/addon-module-template))
//! > - [API](https://crates.io/crates/abstract-api) ([Template (WIP)]())
//! > - [IBC-host](https://crates.io/crates/abstract-ibc-host) ([Template (WIP)]())
//!
//! ```
//!   # use abstract_sdk::{feature_objects::VersionControlContract, base::features::{Identification, AbstractNameService, ModuleIdentification}};
//!   # use cosmwasm_std::{StdResult, Deps, MessageInfo, CosmosMsg, Addr};
//!   # use abstract_sdk::feature_objects::AnsHost;
//!   # use abstract_sdk::AbstractSdkResult;
//!
//!   # pub struct MyContract {
//!   #     
//!   # }
//!   #
//!   # impl Identification for MyContract {
//!   #     fn proxy_address(&self, _deps: Deps) -> AbstractSdkResult<Addr> {
//!   #         Ok(Addr::unchecked("just_an_example"))
//!   #     }
//!   # }
//!   # impl ModuleIdentification for MyContract {
//!   #     fn module_id(&self) -> &'static str { "my_contract" }
//!   # }
//!
//!   # impl AbstractNameService for MyContract {
//!   #     fn ans_host(&self, _deps: Deps) -> AbstractSdkResult<AnsHost> {
//!   #         Ok(AnsHost{address: Addr::unchecked("just_an_example")})
//!   #     }
//!   # }
//!   use abstract_sdk::TransferInterface;
//!
//!   fn forward_deposit(deps: Deps, my_contract: MyContract, message_info: MessageInfo) -> StdResult<CosmosMsg> {
//!       let send_deposit_to_vault_msg = my_contract.bank(deps).deposit_coins(message_info.funds)?;
//!       Ok(send_deposit_to_vault_msg)
//!   }
//!   #
//!   # fn main() {}
//!   ```

pub type AbstractSdkResult<T> = Result<T, crate::error::AbstractSdkError>;

pub extern crate abstract_macros as macros;
pub extern crate abstract_os as os;

mod ans_resolve;
pub mod apis;
pub mod base;
mod error;
pub mod feature_objects;
pub mod helpers;

pub use error::AbstractSdkError;
pub use error::EndpointError;

pub use crate::apis::{
    bank::TransferInterface, execution::Execution, ibc::IbcInterface, modules::ModuleInterface,
    vault::VaultInterface, verify::OsVerification, version_registry::ModuleRegistryInterface,
};
pub use ans_resolve::Resolve;

pub mod namespaces {
    pub use abstract_os::objects::common_namespace::*;
}

pub mod register {
    pub use abstract_os::registry::*;
}
