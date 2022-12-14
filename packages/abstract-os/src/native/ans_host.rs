//! # AnsHost
//!
//! `abstract_os::ans_host` stores chain-specific contract addresses.
//!
//! ## Description
//! Contract and asset addresses are stored on the ans_host contract and are retrievable trough smart or raw queries.

use cosmwasm_schema::QueryResponses;
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use crate::objects::{
    asset_entry::AssetEntry,
    contract_entry::{ContractEntry, UncheckedContractEntry},
    ChannelEntry, UncheckedChannelEntry,
};

/// AnsHost state details
pub mod state {
    use cosmwasm_std::Addr;
    use cw_asset::AssetInfo;
    use cw_controllers::Admin;
    use cw_storage_plus::Map;

    use crate::objects::{
        asset_entry::AssetEntry, common_namespace::ADMIN_NAMESPACE, contract_entry::ContractEntry,
        ChannelEntry,
    };

    /// Admin address store
    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    /// Stores name and address of tokens and pairs
    /// LP token pairs are stored alphabetically
    pub const ASSET_ADDRESSES: Map<AssetEntry, AssetInfo> = Map::new("assets");

    /// Stores contract addresses
    /// Pairs are stored here as (dex_name, pair_id)
    /// pair_id is "asset1_asset2" where the asset names are sorted alphabetically.
    pub const CONTRACT_ADDRESSES: Map<ContractEntry, Addr> = Map::new("contracts");

    /// stores channel-ids
    pub const CHANNELS: Map<ChannelEntry, String> = Map::new("channels");
}

/// AnsHost Instantiate msg
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

/// AnsHost Execute msg
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Updates the contract addressbook
    UpdateContractAddresses {
        // Contracts to update or add
        to_add: Vec<(UncheckedContractEntry, String)>,
        // Contracts to remove
        to_remove: Vec<UncheckedContractEntry>,
    },
    /// Updates the Asset addressbook
    UpdateAssetAddresses {
        // Assets to update or add
        to_add: Vec<(String, AssetInfoUnchecked)>,
        // Assets to remove
        to_remove: Vec<String>,
    },
    /// Updates the Asset addressbook
    UpdateChannels {
        // Assets to update or add
        to_add: Vec<(UncheckedChannelEntry, String)>,
        // Assets to remove
        to_remove: Vec<UncheckedChannelEntry>,
    },
    /// Sets a new Admin
    SetAdmin { admin: String },
}

/// AnsHost smart-query
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    /// Queries assets based on name
    /// returns [`AssetsResponse`]
    #[returns(AssetsResponse)]
    Assets {
        // Names of assets to query
        names: Vec<String>,
    },
    /// Page over assets
    /// returns [`AssetListResponse`]
    #[returns(AssetListResponse)]
    AssetList {
        page_token: Option<String>,
        page_size: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ContractsResponse`]
    #[returns(ContractsResponse)]
    Contracts {
        // Project and contract names of contracts to query
        names: Vec<ContractEntry>,
    },
    /// Page over contracts
    /// returns [`ContractListResponse`]
    #[returns(ContractListResponse)]
    ContractList {
        page_token: Option<ContractEntry>,
        page_size: Option<u8>,
    },
    /// Queries contracts based on name
    /// returns [`ChannelsResponse`]
    #[returns(ChannelsResponse)]
    Channels {
        // Project and contract names of contracts to query
        names: Vec<ChannelEntry>,
    },
    /// Page over contracts
    /// returns [`ChannelListResponse`]
    #[returns(ChannelListResponse)]
    ChannelList {
        page_token: Option<ChannelEntry>,
        page_size: Option<u8>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetsResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(AssetEntry, AssetInfo)>,
}

/// Query response
#[cosmwasm_schema::cw_serde]
pub struct AssetListResponse {
    /// Assets (name, assetinfo)
    pub assets: Vec<(AssetEntry, AssetInfo)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractsResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(ContractEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ContractListResponse {
    /// Contracts (name, address)
    pub contracts: Vec<(ContractEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelsResponse {
    pub channels: Vec<(ChannelEntry, String)>,
}

#[cosmwasm_schema::cw_serde]
pub struct ChannelListResponse {
    pub channels: Vec<(ChannelEntry, String)>,
}
