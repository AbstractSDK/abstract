//! # Module Factory
//!
//! `abstract_os::module_factory` is a native contract that handles instantiation and migration of os modules.
//!
//! ## Description  
//! This contract is instantiated by Abstract and only used internally. Adding or upgrading modules is done using the [`crate::manager::ExecuteMsg`] endpoint.  
pub mod state {
    use crate::{
        objects::module::{Module, ModuleInfo},
        version_control::Core,
    };
    use cosmwasm_std::{Addr, Binary};
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
    pub struct Config {
        pub version_control_address: Addr,
        pub memory_address: Addr,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct Context {
        pub core: Option<Core>,
        pub module: Option<Module>,
    }

    pub const ADMIN: Admin = Admin::new("admin");
    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const CONTEXT: Item<Context> = Item::new("\u{0}{7}context");
    pub const MODULE_INIT_BINARIES: Map<ModuleInfo, Binary> = Map::new("module_init_binaries");
}

use crate::{
    objects::module::{Module, ModuleInfo},
    version_control::Core,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Binary;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Version control address used to get code-ids and register OS
    pub version_control_address: String,
    /// Memory address
    pub memory_address: String,
}

#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        admin: Option<String>,
        memory_address: Option<String>,
        version_control_address: Option<String>,
    },
    /// Creates the core contracts for the OS
    CreateModule {
        /// Module details
        module: Module,
        init_msg: Option<Binary>,
    },
    UpdateFactoryBinaryMsgs {
        to_add: Vec<(ModuleInfo, Binary)>,
        to_remove: Vec<ModuleInfo>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(ContextResponse)]
    Context {},
}

// We define a custom struct for each query response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub memory_address: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ContextResponse {
    pub core: Option<Core>,
    pub module: Option<Module>,
}

/// We currently take no arguments for migrations
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
