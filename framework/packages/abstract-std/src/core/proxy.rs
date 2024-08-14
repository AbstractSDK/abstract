//! # Account Proxy
//!
//! `abstract_std::proxy` hold all the assets associated with the Account instance. It accepts Cosmos messages from whitelisted addresses and executes them.
//!
//! ## Description
//! The proxy is part of the Core Account contracts along with the [`crate::manager`] contract.
//! This contract is responsible for executing Cosmos messages and calculating the value of its internal assets.
//!
//! ## Price Sources
//! [price sources](crate::objects::price_source) are what allow the proxy contract to provide value queries for its assets. It needs to be configured using the [`ExecuteMsg::UpdateAssets`] endpoint.
//! After configuring the price sources [`QueryMsg::TotalValue`] can be called to get the total holding value.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Empty, Uint128};
use cw_asset::{Asset, AssetInfo};

#[allow(unused_imports)]
use crate::{ibc_client::ExecuteMsg as IbcClientMsg, objects::account::AccountId};

pub mod state {
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::Item;

    pub use crate::objects::account::ACCOUNT_ID;
    use crate::objects::common_namespace::ADMIN_NAMESPACE;
    #[cosmwasm_schema::cw_serde]
    pub struct State {
        pub modules: Vec<Addr>,
    }
    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub account_id: AccountId,
    pub ans_host_address: String,
    pub manager_addr: String,
}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Sets the admin
    SetAdmin { admin: String },
    /// Executes the provided messages if sender is whitelisted
    ModuleAction { msgs: Vec<CosmosMsg<Empty>> },
    /// Execute a message and forward the Response data
    ModuleActionWithData { msg: CosmosMsg<Empty> },
    /// Execute IBC action on Client
    IbcAction { msg: IbcClientMsg },
    /// Adds the provided address to whitelisted dapps
    AddModules { modules: Vec<String> },
    /// Removes the provided address from the whitelisted dapps
    RemoveModule { module: String },
}
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Contains the enabled modules
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub modules: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct TokenValueResponse {
    pub value: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseAssetResponse {
    pub base_asset: AssetInfo,
}

/// Query message to external contract to get asset value
#[cosmwasm_schema::cw_serde]
pub struct ValueQueryMsg {
    pub asset: AssetInfo,
    pub amount: Uint128,
}
/// External contract value response
#[cosmwasm_schema::cw_serde]
pub struct ExternalValueResponse {
    pub value: Asset,
}
