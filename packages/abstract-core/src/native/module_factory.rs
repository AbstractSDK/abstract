//! # Module Factory
//!
//! `abstract_core::module_factory` is a native contract that handles instantiation and migration of account modules.
//!
//! ## Description  
//! This contract is instantiated by Abstract and only used internally. Adding or upgrading modules is done using the [`crate::manager::ExecuteMsg`] endpoint.  
pub mod state {
    use crate::{
        objects::{
            common_namespace::ADMIN_NAMESPACE,
            module::{Module, ModuleInfo},
        },
        version_control::AccountBase,
    };
    use cosmwasm_std::{Addr, Binary};
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
    pub struct Config {
        pub version_control_address: Addr,
        pub ans_host_address: Addr,
    }

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
    pub struct Context {
        pub core: Option<AccountBase>,
        pub module: Option<Module>,
    }

    pub const ADMIN: Admin = Admin::new(ADMIN_NAMESPACE);
    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const CONTEXT: Item<Context> = Item::new("\u{0}{7}context");
    pub const MODULE_INIT_BINARIES: Map<&ModuleInfo, Binary> = Map::new("module_init_binaries");
}

use crate::{
    objects::module::{Module, ModuleInfo},
    version_control::AccountBase,
};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Binary;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    /// Version control address used to get code-ids and register Account
    pub version_control_address: String,
    /// AnsHost address
    pub ans_host_address: String,
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "boot", derive(boot_core::ExecuteFns))]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        admin: Option<String>,
        ans_host_address: Option<String>,
        version_control_address: Option<String>,
    },
    /// Installs a module on the Account
    InstallModule {
        // Module details
        module: ModuleInfo,
        init_msg: Option<Binary>,
    },
    UpdateFactoryBinaryMsgs {
        to_add: Vec<(ModuleInfo, Binary)>,
        to_remove: Vec<ModuleInfo>,
    },
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[cfg_attr(feature = "boot", derive(boot_core::QueryFns))]
pub enum QueryMsg {
    /// Get the configuration for the module factory.
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Get the installation context of the module factory.
    /// Returns [`ContextResponse`]
    #[returns(ContextResponse)]
    Context {},
}

// We define a custom struct for each query response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub owner: String,
    pub ans_host_address: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct ContextResponse {
    pub account: Option<AccountBase>,
    pub module: Option<Module>,
}

/// We currently take no arguments for migrations
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
