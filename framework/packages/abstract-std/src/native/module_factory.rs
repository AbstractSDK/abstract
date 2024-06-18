//! # Module Factory
//!
//! `abstract_std::module_factory` is a native contract that handles instantiation and migration of account modules.
//!
//! ## Description  
//! This contract is instantiated by Abstract and only used internally. Adding or upgrading modules is done using the [`crate::manager::ExecuteMsg`] endpoint.  
pub mod state {

    use cosmwasm_std::{Addr, Binary};
    use cw_storage_plus::{Item, Map};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    use crate::{objects::module::ModuleInfo, version_control::AccountBase};

    #[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, JsonSchema)]
    pub struct Config {
        pub version_control_address: Addr,
        pub ans_host_address: Addr,
    }

    pub const CONFIG: Item<Config> = Item::new("\u{0}{5}config");
    pub const MODULE_INIT_BINARIES: Map<&ModuleInfo, Binary> = Map::new("module_init_binaries");
    pub const CURRENT_BASE: Item<AccountBase> = Item::new("cur_manager");
}

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Binary, Coin};

use crate::objects::module::ModuleInfo;

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub admin: String,
    /// Version control address used to get code-ids and register Account
    pub version_control_address: String,
    /// AnsHost address
    pub ans_host_address: String,
}

/// Module Factory Execute messages
#[cw_ownable::cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    /// Update config
    UpdateConfig {
        ans_host_address: Option<String>,
        version_control_address: Option<String>,
    },
    /// Install modules
    InstallModules {
        modules: Vec<FactoryModuleInstallConfig>,
        salt: Binary,
    },
    UpdateFactoryBinaryMsgs {
        to_add: Vec<(ModuleInfo, Binary)>,
        to_remove: Vec<ModuleInfo>,
    },
}

/// Module info, init message and salt
#[non_exhaustive]
#[cosmwasm_schema::cw_serde]
pub struct FactoryModuleInstallConfig {
    pub module: ModuleInfo,
    pub init_msg: Option<Binary>,
}

impl FactoryModuleInstallConfig {
    pub fn new(module: ModuleInfo, init_msg: Option<Binary>) -> Self {
        Self { module, init_msg }
    }
}

/// Module factory query messages
#[cw_ownable::cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Get the configuration for the module factory.
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Simulate install module cost
    /// Returns [`SimulateInstallModulesResponse`]
    #[returns(SimulateInstallModulesResponse)]
    SimulateInstallModules { modules: Vec<ModuleInfo> },
}

/// Module factory config response
#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_address: Addr,
    pub version_control_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct SimulateInstallModulesResponse {
    pub total_required_funds: Vec<Coin>,
    /// Funds transferred to the module creator
    pub monetization_funds: Vec<(String, Coin)>,
    /// Funds transferred to the module contract at instantiation
    pub initialization_funds: Vec<(String, Vec<Coin>)>,
}

/// We currently take no arguments for migrations
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg {}
