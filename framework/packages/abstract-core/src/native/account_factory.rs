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
    use cosmwasm_std::Addr;
    use cw_storage_plus::Item;
    use serde::{Deserialize, Serialize};

    use crate::objects::{account_id::AccountId, module::Module};

    /// Account Factory configuration
    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_contract: Addr,
        pub ans_host_contract: Addr,
        pub module_factory_address: Addr,
        pub next_account_id: AccountId,
    }

    /// Account Factory context for post-[`crate::abstract_manager`] [`crate::abstract_proxy`] creation
    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Context {
        pub account_manager_address: Option<Addr>,
        pub manager_module: Option<Module>,
        pub proxy_module: Option<Module>,
    }

    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const CONTEXT: Item<Context> = Item::new("\u{0}{6}context");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Addr;

use crate::objects::{account_id::AccountId, gov_type::GovernanceDetails};

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
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    CreateAccount {
        // Governance details
        governance: GovernanceDetails<String>,
        // Account name
        name: String,
        // Account description
        description: Option<String>,
        // Account link
        link: Option<String>,
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
    pub next_account_id: AccountId,
}

/// Account Factory migrate messages
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
