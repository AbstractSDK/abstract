//! # State and Message Objects
//! This module contains all the structs and enums used in contract state-storage or contained in contract interaction.

pub(crate) mod ans_asset;
pub mod ans_host;
pub mod common_namespace;
pub mod version_control;

mod entry;
pub mod nested_admin;
pub mod oracle;
pub mod pool;
pub mod salt;

pub use pool::*;

pub mod account;
pub mod dependency;
pub mod deposit_info;
pub mod deposit_manager;
pub mod fee;
pub mod gov_type;
pub mod module;
pub mod module_reference;
pub mod module_version;
pub mod namespace;
pub mod paged_map;
pub mod price_source;
pub mod time_weighted_average;
pub(crate) mod truncated_chain_id;
pub mod validation;
pub mod voting;

pub use account::{AccountId, ABSTRACT_ACCOUNT_ID};
pub use ans_asset::AnsAsset;
pub use entry::{
    ans_entry_convertor::AnsEntryConvertor,
    asset_entry::AssetEntry,
    channel_entry::{ChannelEntry, UncheckedChannelEntry},
    contract_entry::{ContractEntry, UncheckedContractEntry},
    dex_asset_pairing::DexAssetPairing,
    lp_token::{DexName, LpToken},
};
pub use truncated_chain_id::TruncatedChainId;

pub mod chain_name {
    use super::TruncatedChainId;

    // Type name `ChainName` was not suitable name for the type
    #[deprecated = "Use TruncatedChainId instead"]
    pub type ChainName = TruncatedChainId;
}
