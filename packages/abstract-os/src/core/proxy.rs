//! # OS Proxy
//!
//! `abstract_os::proxy` hold all the assets associated with the OS instance. It accepts Cosmos messages from whitelisted addresses and executes them.
//!
//! ## Description
//! The proxy is part of the Core OS contracts along with the [`crate::manager`] contract.
//! This contract is responsible for executing Cosmos messages and calculating the value of its internal assets.
//!
//! ## Proxy assets
//! [Proxy assets](crate::objects::proxy_asset) are what allow the proxy contract to provide value queries for its assets. It needs to be configured using the [`ExecuteMsg::UpdateAssets`] endpoint.
//! After configuring the proxy assets [`QueryMsg::TotalValue`] can be called to get the total holding value.

use crate::ibc_client::ExecuteMsg as IbcClientMsg;
use crate::objects::{
    proxy_asset::{ProxyAsset, UncheckedProxyAsset},
    AssetEntry,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, Empty, Uint128};

pub mod state {
    pub use crate::objects::core::OS_ID;
    use cw_controllers::Admin;

    use cosmwasm_std::Addr;
    use cw_storage_plus::{Item, Map};

    use crate::objects::{
        ans_host::AnsHost, asset_entry::AssetEntry, common_namespace::ADMIN_NAMESPACE,
        proxy_asset::ProxyAsset,
    };
    #[cosmwasm_schema::cw_serde]
    pub struct State {
        pub modules: Vec<Addr>,
    }
    pub const ANS_HOST: Item<AnsHost> = Item::new("\u{0}{6}ans_host");
    pub const STATE: Item<State> = Item::new("\u{0}{5}state");
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const VAULT_ASSETS: Map<AssetEntry, ProxyAsset> = Map::new("proxy_assets");
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub os_id: u32,
    pub ans_host_address: String,
}

// hot fix
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
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
        to_add: Vec<UncheckedProxyAsset>,
        to_remove: Vec<String>,
    },
}
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns the total value of all held assets
    /// [`TotalValueResponse`]
    #[returns(TotalValueResponse)]
    TotalValue {},
    /// Returns the value of amount OR one token of a specific asset
    /// [`TokenValueResponse`]
    #[returns(TokenValueResponse)]
    TokenValue {
        identifier: String,
        amount: Option<Uint128>,
    },
    /// Returns the value of one specific asset
    /// [`HoldingValueResponse`]
    #[returns(HoldingValueResponse)]
    HoldingValue { identifier: String },
    /// Returns the amount of specified tokens this contract holds
    /// [`HoldingAmountResponse`]
    #[returns(HoldingAmountResponse)]
    HoldingAmount { identifier: String },
    /// Returns the VAULT_ASSETS value for the specified key
    /// [`AssetConfigResponse`]
    #[returns(AssetConfigResponse)]
    AssetConfig { identifier: String },
    /// Returns [`AssetsResponse`]
    #[returns(AssetsResponse)]
    Assets {
        page_token: Option<String>,
        page_size: Option<u8>,
    },
    /// Returns [`ValidityResponse`]
    #[returns(ValidityResponse)]
    CheckValidity {},
    /// Returns [`BaseAssetResponse`]
    #[returns(BaseAssetResponse)]
    BaseAsset {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub modules: Vec<String>,
}

#[cosmwasm_schema::cw_serde]
pub struct TotalValueResponse {
    pub value: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct TokenValueResponse {
    pub value: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct HoldingValueResponse {
    pub value: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct ValidityResponse {
    /// Assets that have unresolvable dependencies in their value calculation
    pub unresolvable_assets: Option<Vec<AssetEntry>>,
    /// Assets that are missing in the VAULT_ASSET map which caused some assets to be unresolvable.
    pub missing_dependencies: Option<Vec<AssetEntry>>,
}

#[cosmwasm_schema::cw_serde]
pub struct BaseAssetResponse {
    pub base_asset: ProxyAsset,
}

#[cosmwasm_schema::cw_serde]
pub struct HoldingAmountResponse {
    pub amount: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct AssetConfigResponse {
    pub proxy_asset: ProxyAsset,
}

#[cosmwasm_schema::cw_serde]
pub struct AssetsResponse {
    pub assets: Vec<(AssetEntry, ProxyAsset)>,
}

/// Query message to external contract to get asset value
#[cosmwasm_schema::cw_serde]

pub struct ValueQueryMsg {
    pub asset: AssetEntry,
    pub amount: Uint128,
}
/// External contract value response
#[cosmwasm_schema::cw_serde]
pub struct ExternalValueResponse {
    pub value: Uint128,
}
