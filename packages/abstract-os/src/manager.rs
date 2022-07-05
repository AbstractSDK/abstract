pub mod state {
    pub use crate::objects::core::OS_ID;
    use cosmwasm_std::Addr;
    use cw_controllers::Admin;
    use cw_storage_plus::{Item, Map};
    use schemars::JsonSchema;
    use serde::{Deserialize, Serialize};

    pub type Subscribed = bool;
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct Config {
        pub version_control_address: Addr,
        pub module_factory_address: Addr,
        pub subscription_address: Addr,
    }
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    pub struct OsInfo {
        pub name: String,
        pub governance_type: String,
        pub chain_id: String,
        pub description: Option<String>,
        pub link: Option<String>,
    }

    pub const STATUS: Item<Subscribed> = Item::new("\u{0}{6}status");
    pub const CONFIG: Item<Config> = Item::new("\u{0}{6}config");
    /// Info about the OS
    pub const INFO: Item<OsInfo> = Item::new("\u{0}{4}info");
    /// Contract Admin
    pub const ADMIN: Admin = Admin::new("admin");
    pub const ROOT: Admin = Admin::new("root");
    pub const OS_MODULES: Map<&str, Addr> = Map::new("os_modules");
}

use cosmwasm_std::{Binary, Uint64};
use cw2::ContractVersion;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::modules::Module;

use self::state::OsInfo;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub os_id: u32,
    pub root_user: String,
    pub version_control_address: String,
    pub module_factory_address: String,
    pub subscription_address: Option<String>,
    pub chain_id: String,
    pub governance_type: String,
    pub os_name: String,
    pub description: Option<String>,
    pub link: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    /// Updates the Modules
    UpdateModuleAddresses {
        to_add: Option<Vec<(String, String)>>,
        to_remove: Option<Vec<String>>,
    },
    /// Sets a new Admin
    SetAdmin {
        admin: String,
        governance_type: Option<String>,
    },
    /// Create module using module factory
    CreateModule {
        module: Module,
        init_msg: Option<Binary>,
    },
    /// Register a module after creation
    RegisterModule { module_addr: String, module: Module },
    /// Remove a module
    RemoveModule { module_name: String },
    /// Forward configuration message to module
    ExecOnModule {
        module_name: String,
        exec_msg: Binary,
    },
    /// Update contract configuration
    UpdateConfig {
        vc_addr: Option<String>,
        root: Option<String>,
    },
    Upgrade {
        module: Module,
        migrate_msg: Option<Binary>,
    },
    /// Suspend manager contract
    SuspendOs { new_status: bool },
    /// Update info
    UpdateInfo {
        os_name: Option<String>,
        description: Option<String>,
        link: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    /// Returns [`QueryModuleVersionsResponse`]
    QueryModuleVersions { names: Vec<String> },
    /// Returns [`QueryModuleAddressesResponse`]
    QueryModuleAddresses { names: Vec<String> },
    /// Returns [`QueryModuleInfosResponse`]
    QueryModuleInfos {
        last_module_name: Option<String>,
        iter_limit: Option<u8>,
    },
    /// Returns [`QueryConfigResponse`]
    QueryConfig {},
    /// Returns [`QueryInfoResponse`]
    QueryInfo {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryModuleVersionsResponse {
    pub versions: Vec<ContractVersion>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryModuleAddressesResponse {
    pub modules: Vec<(String, String)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryConfigResponse {
    pub root: String,
    pub version_control_address: String,
    pub module_factory_address: String,
    pub os_id: Uint64,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryInfoResponse {
    pub info: OsInfo,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ManagerModuleInfo {
    pub name: String,
    pub version: ContractVersion,
    pub address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct QueryModuleInfosResponse {
    pub module_infos: Vec<ManagerModuleInfo>,
}
