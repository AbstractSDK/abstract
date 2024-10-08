// Remove deprecated if we move to cosmwasm_2_0 feature

pub mod adapter;
pub mod app;
pub mod bank;
pub mod execution;
pub mod ibc;
pub mod modules;
pub mod respond;
mod splitter;
mod traits;
pub mod verify;
pub mod version_registry;

pub use traits::{AbstractApi, ApiIdentification};

#[cfg(feature = "stargate")]
pub mod authz;
#[cfg(feature = "stargate")]
pub mod distribution;
#[cfg(feature = "stargate")]
pub mod feegrant;
#[cfg(feature = "stargate")]
pub mod stargate;

#[cfg(feature = "stargate")]
pub(crate) fn stargate_msg(
    type_url: String,
    value: cosmwasm_std::Binary,
) -> cosmwasm_std::CosmosMsg {
    #[allow(deprecated)]
    cosmwasm_std::CosmosMsg::Stargate { type_url, value }
}
