//! # Account Factory
//!
//! `abstract_core::account_factory` handles Account creation and registration.
//!
//! ## Description
//! The Account factory instantiates a new Account instance and registers it with the [`crate::version_control`] contract. It then forwards the payment to the main account's subscription module.  
//! ## Create a new Account
//! Call [`ExecuteMsg::CreateAccount`] on this contract along with a [`crate::objects::gov_type`] and name you'd like to display on your Account.
//!
pub mod state {
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::Item;

    use serde::{Deserialize, Serialize};

    use crate::objects::{common_namespace::ADMIN_NAMESPACE, core::AccountId};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_contract: Addr,
        pub ans_host_contract: Addr,
        pub module_factory_address: Addr,
        pub next_account_id: AccountId,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Context {
        pub account_manager_address: Addr,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const CONTEXT: Item<Context> = Item::new("\u{0}{6}context");
}

use crate::objects::{core::AccountId, gov_type::GovernanceDetails};
use cosmwasm_schema::QueryResponses;
use cw20::Cw20ReceiveMsg;

/// Msg used on instantiation
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Version control contract used to get code-ids and register Account
    pub version_control_address: String,
    /// AnsHost contract
    pub ans_host_address: String,
    /// AnsHosts of module factory. Used for instantiating manager.
    pub module_factory_address: String,
}

/// Execute function entrypoint.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Handler called by the CW-20 contract on a send-call
    Receive(Cw20ReceiveMsg),
    /// Update config
    UpdateConfig {
        // New admin
        admin: Option<String>,
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
        name: String,
        description: Option<String>,
        link: Option<String>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

// We define a custom struct for each query response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub ans_host_contract: String,
    pub version_control_contract: String,
    pub module_factory_address: String,
    pub next_account_id: AccountId,
}

/// We currently take no arguments for migrations
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
