//! # Account Factory
//!
//! `abstract_core::account_factory` handles Account creation and registration.
//!
//! ## Description
//! The Account factory instantiates a new Account instance and registers it with the [`crate::version_control`] contract.  
//! ## Create a new Account
//! Call [`ExecuteMsg::CreateAccount`] on this contract along with a [`crate::objects::gov_type`] and name you'd like to display on your Account.
//!
pub mod state {
    use cosmwasm_std::{Addr, Coin};
    use cw_storage_plus::Item;
    use serde::{Deserialize, Serialize};

    use crate::{
        native::module_factory::ModuleInstallConfig,
        objects::{
            account::{AccountId, AccountSequence},
            gov_type::GovernanceDetails,
            module::Module,
            AssetEntry,
        }, manager::ManagerModuleInstall,
    };

    /// Account Factory configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_contract: Addr,
        pub ans_host_contract: Addr,
        pub module_factory_address: Addr,
        pub ibc_host: Option<Addr>,
    }

    /// Account Factory context for post-[`crate::abstract_manager`] [`crate::abstract_proxy`] creation
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Context {
        pub account_proxy_address: Option<Addr>,
        pub manager_module: Option<Module>,
        pub proxy_module: Option<Module>,
        pub account_id: AccountId,

        pub additional_config: AdditionalContextConfig,
        pub install_modules: Vec<ManagerModuleInstall>,
        pub funds_for_install: Vec<Coin>,
    }

    /// Account Factory additional config context for post-[`crate::abstract_manager`] [`crate::abstract_proxy`] creation
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct AdditionalContextConfig {
        pub namespace: Option<String>,
        pub base_asset: Option<AssetEntry>,
        pub name: String,
        pub description: Option<String>,
        pub link: Option<String>,
        pub owner: GovernanceDetails<String>,
    }

    pub const CONFIG: Item<Config> = Item::new("cfg");
    pub const CONTEXT: Item<Context> = Item::new("contxt");
    pub const LOCAL_ACCOUNT_SEQUENCE: Item<AccountSequence> = Item::new("acseq");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

use crate::{
    native::module_factory::ModuleInstallConfig,
    objects::{
        account::{AccountId, AccountSequence, AccountTrace},
        gov_type::GovernanceDetails,
        AssetEntry,
    }, manager::ManagerModuleInstall,
};

/// Msg used on instantiation
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Admin of the contract
    pub admin: String,
    /// Version control contract used to get code-ids and register Account
    pub version_control_address: String,
    /// AnsHost contract
    pub ans_host_address: String,
    /// AnsHosts of module factory. Used for instantiating manager.
    pub module_factory_address: String,
}

/// Account Factory execute messages
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        // New ans_host contract
        ans_host_contract: Option<String>,
        // New version control contract
        version_control_contract: Option<String>,
        // New module factory contract
        module_factory_address: Option<String>,
        // New ibc host contract
        ibc_host: Option<String>,
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    CreateAccount {
        // Governance details
        governance: GovernanceDetails<String>,
        // Account name
        name: String,
        // Optionally specify a base asset for the account
        base_asset: Option<AssetEntry>,
        // Account description
        description: Option<String>,
        // Account link
        link: Option<String>,
        /// Account id on the remote chain. Will create a new id (by incrementing), if not specified
        account_id: Option<AccountId>,
        // optionally specify a namespace for the account
        namespace: Option<String>,
        // Provide list of module to install after account creation
        install_modules: Vec<ManagerModuleInstall>,
    },
}

/// Account Factory query messages
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

/// Account Factory config response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_contract: Addr,
    pub version_control_contract: Addr,
    pub module_factory_address: Addr,
    pub ibc_host: Option<Addr>,
    pub local_account_sequence: AccountSequence,
}

/// Sequence numbers for each origin.
#[cosmwasm_schema::cw_serde]
pub struct SequencesResponse {
    pub sequences: Vec<(AccountTrace, AccountSequence)>,
}

#[cosmwasm_schema::cw_serde]
pub struct SequenceResponse {
    pub sequence: AccountSequence,
}

/// Account Factory migrate messages
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
