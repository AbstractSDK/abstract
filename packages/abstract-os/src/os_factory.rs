//! # Os Factory
//!
//! `abstract_os::os_factory` handles OS creation and registration.
//!
//! ## Description
//! The OS factory instantiates a new OS instance and registeres it with the [`crate::version_control`] contract. It then forwards the payment to the main os's subscription module.  
//! ## Create a new OS
//! Call [`ExecuteMsg::CreateOs`] on this contract along with a [`crate::objects::gov_type`] and name you'd like to display on your OS.
//!
pub mod state {
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::Item;

    use serde::{Deserialize, Serialize};

    #[cosmwasm_schema::cw_serde]
    pub struct Config {
        pub version_control_contract: Addr,
        pub memory_contract: Addr,
        pub module_factory_address: Addr,
        pub subscription_address: Option<Addr>,
        pub next_os_id: u32,
    }

    #[derive(Serialize, Deserialize, Clone, Debug)]
    pub struct Context {
        pub os_manager_address: Addr,
    }

    pub const ADMIN: Admin = Admin::new("admin");
    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const CONTEXT: Item<Context> = Item::new("\u{0}{6}context");
}

use crate::objects::gov_type::GovernanceDetails;
use cosmwasm_schema::QueryResponses;
use cw20::Cw20ReceiveMsg;

/// Msg used on instantiation
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Version control contract used to get code-ids and register OS
    pub version_control_address: String,
    /// Memory contract
    pub memory_address: String,
    /// Address of module factory. Used for instantiating manager.
    pub module_factory_address: String,
}

/// Execute function entrypoint.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Handler called by the CW-20 contract on a send-call
    Receive(Cw20ReceiveMsg),
    /// Update config
    UpdateConfig {
        /// New admin
        admin: Option<String>,
        /// New memory contract
        memory_contract: Option<String>,
        /// New version control contract
        version_control_contract: Option<String>,
        /// New module factory contract
        module_factory_address: Option<String>,
        /// New subscription contract
        subscription_address: Option<String>,
    },
    /// Creates the core contracts and sets the permissions.
    /// [`crate::manager`] and [`crate::proxy`]
    CreateOs {
        /// Governance details
        /// Use [`crate::objects::GovernanceDetails::Monarchy`] to use a custom governance modal.
        /// TODO: add support for other types of gov.
        governance: GovernanceDetails,
        name: String,
        description: Option<String>,
        link: Option<String>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
}

// We define a custom struct for each query response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub memory_contract: String,
    pub version_control_contract: String,
    pub module_factory_address: String,
    pub subscription_address: Option<String>,
    pub next_os_id: u32,
}

/// We currently take no arguments for migrations
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
