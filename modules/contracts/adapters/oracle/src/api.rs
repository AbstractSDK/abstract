use crate::ORACLE_ADAPTER_ID;
use abstract_core::objects::{module::ModuleId, AnsAsset, AssetEntry, PoolAddress};
use abstract_oracle_standard::msg::{OracleExecuteMsg, OracleQueryMsg, ProviderName};
use abstract_sdk::{
    features::{AccountIdentification, Dependencies, ModuleIdentification},
    AbstractSdkResult, AdapterInterface,
};
use cosmwasm_schema::serde::de::DeserializeOwned;
use cosmwasm_std::{CosmosMsg, Decimal, Deps};
use cw_asset::{Asset, AssetInfo, AssetInfoBase};

// TODO:

// API for Abstract SDK users
/// Interact with the oracle adapter in your module.
// pub trait OracleInterface: AccountIdentification + Dependencies + ModuleIdentification {
//     /// Construct a new oracle interface.
//     fn oracle<'a>(&'a self, deps: Deps<'a>, provider: ProviderName) -> OracleApi<Self> {
//         OracleApi {
//             base: self,
//             deps,
//             name,
//             module_id: ORACLE_ADAPTER_ID,
//         }
//     }
// }

// impl<T: AccountIdentification + Dependencies + ModuleIdentification> OracleInterface for T {}
#[cfg(test)]
mod test {}
