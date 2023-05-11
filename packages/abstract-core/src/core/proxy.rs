//! # Account Proxy
//!
//! `abstract_core::proxy` hold all the assets associated with the Account instance. It accepts Cosmos messages from whitelisted addresses and executes them.
//!
//! ## Description
//! The proxy is part of the Core Account contracts along with the [`crate::manager`] contract.
//! This contract is responsible for executing Cosmos messages and calculating the value of its internal assets.
//!
//! ## Price Sources
//! [price sources](crate::objects::price_source) are what allow the proxy contract to provide value queries for its assets. It needs to be configured using the [`ExecuteMsg::UpdateAssets`] endpoint.
//! After configuring the price sources [`QueryMsg::TotalValue`] can be called to get the total holding value.

#[allow(unused_imports)]
use crate::{
    ibc_client::ExecuteMsg as IbcClientMsg,
    objects::{
        account_id::AccountId,
        oracle::{AccountValue, Complexity},
        price_source::{PriceSource, UncheckedPriceSource},
        AssetEntry,
    },
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Empty, Uint128};
use cw_asset::{Asset, AssetInfo};

pub mod state {
    pub use crate::objects::account_id::ACCOUNT_ID;
    use cw_controllers::Admin;

    use cosmwasm_std::Addr;
    use cw_storage_plus::Item;

    use crate::objects::{ans_host::AnsHost, common_namespace::ADMIN_NAMESPACE};
    #[cosmwasm_schema::cw_serde]
    pub struct State {
        pub modules: Vec<Addr>,
    }
    pub const ANS_HOST: Item<AnsHost> = Item::new("\u{0}{6}ans_host");
    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub account_id: AccountId,
    pub ans_host_address: String,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Sets the admin
    SetAdmin { admin: String },
    /// Executes the provided messages if sender is whitelisted
    ModuleAction { msgs: Vec<CosmosMsg<Empty>> },
    /// Execute IBC action on Client
    IbcAction { msgs: Vec<IbcClientMsg> },
    /// Adds the provided address to whitelisted dapps
    AddModule { module: String },
    /// Removes the provided address from the whitelisted dapps
    RemoveModule { module: String },
    /// Updates the VAULT_ASSETS map
    UpdateAssets {
        to_add: Vec<(AssetEntry, UncheckedPriceSource)>,
        to_remove: Vec<AssetEntry>,
    },
}
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    /// Contains the enabled modules
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns the total value of the assets held by this account
    /// [`AccountValue`]
    #[returns(AccountValue)]
    TotalValue {},
    /// Returns the value of one token with an optional amount set.
    /// If amount is not set, the account's balance of the token is used.
    /// [`TokenValueResponse`]
    #[returns(TokenValueResponse)]
    TokenValue { identifier: AssetEntry },
    /// Returns the amount of specified tokens this contract holds
    /// [`HoldingAmountResponse`]
    #[returns(HoldingAmountResponse)]
    HoldingAmount { identifier: AssetEntry },
    /// Returns the oracle configuration value for the specified key
    /// [`AssetConfigResponse`]
    #[returns(AssetConfigResponse)]
    AssetConfig { identifier: AssetEntry },
    /// Returns [`AssetsConfigResponse`]
    /// Human readable
    #[returns(AssetsConfigResponse)]
    AssetsConfig {
        start_after: Option<AssetEntry>,
        limit: Option<u8>,
    },
    /// Returns [`AssetsInfoResponse`]
    /// Not human readable
    #[returns(AssetsInfoResponse)]
    AssetsInfo {
        start_after: Option<AssetInfo>,
        limit: Option<u8>,
    },
    /// Returns [`BaseAssetResponse`]
    #[returns(BaseAssetResponse)]
    BaseAsset {},
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

#[cosmwasm_schema::cw_serde]
pub struct HoldingAmountResponse {
    pub amount: Uint128,
}

/// Human readable config for a single asset
#[cosmwasm_schema::cw_serde]
pub struct AssetConfigResponse {
    pub price_source: UncheckedPriceSource,
}

/// non-human readable asset configuration
#[cosmwasm_schema::cw_serde]
pub struct AssetsInfoResponse {
    pub assets: Vec<(AssetInfo, OracleAsset)>,
}

/// Human readable asset configuration
#[cosmwasm_schema::cw_serde]
pub struct AssetsConfigResponse {
    pub assets: Vec<(AssetEntry, UncheckedPriceSource)>,
}

#[cosmwasm_schema::cw_serde]
pub struct OracleAsset {
    pub price_source: PriceSource,
    pub complexity: Complexity,
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
