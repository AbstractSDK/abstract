/// namespace key for State Item
pub const BASE_STATE: &str = "base_state";
/// namespace for contract Admin
pub const ADMIN_NAMESPACE: &str = "admin";
/// storage key for cw_ownable::Ownership
pub const OWNERSHIP_STORAGE_KEY: &str = "ownership";
/// storage key for ModuleData
pub const MODULE_STORAGE_KEY: &str = "mod";

pub mod account {
    pub const SUSPENSION_STATUS: &str = "aa";
    pub const CONFIG: &str = "ab";
    pub const INFO: &str = "ac";
    pub const ACCOUNT_MODULES: &str = "ad";
    pub const DEPENDENTS: &str = "ae";
    pub const SUB_ACCOUNTS: &str = "af";
    pub const WHITELISTED_MODULES: &str = "ag";
    pub const ACCOUNT_ID: &str = "ah";
    pub const INSTALL_MODULES_CONTEXT: &str = "ai";
    pub const MIGRATE_CONTEXT: &str = "aj";
}

pub mod ans_host {
    pub const CONFIG: &str = "ba";
    pub const ASSET_ADDRESSES: &str = "bb";
    pub const REV_ASSET_ADDRESSES: &str = "bc";
    pub const CONTRACT_ADDRESSES: &str = "bd";
    pub const CHANNELS: &str = "be";
    pub const REGISTERED_DEXES: &str = "bf";
    pub const ASSET_PAIRINGS: &str = "bg";
    pub const POOL_METADATA: &str = "bh";
}

pub mod version_control {
    pub const CONFIG: &str = "ca";
    pub const PENDING_MODULES: &str = "cb";
    pub const REGISTERED_MODULES: &str = "cc";
    pub const STANDALONE_INFOS: &str = "cd";
    pub const SERVICE_INFOS: &str = "ce";
    pub const YANKED_MODULES: &str = "cf";
    pub const MODULE_CONFIG: &str = "cg";
    pub const MODULE_DEFAULT_CONFIG: &str = "ch";
    pub const ACCOUNT_ADDRESSES: &str = "ci";
    pub const LOCAL_ACCOUNT_SEQUENCE: &str = "cj";
}

pub mod module_factory {
    pub const CONFIG: &str = "da";
    pub const CURRENT_BASE: &str = "db";
}
pub mod ibc_client {
    pub const IBC_INFRA: &str = "ea";
    pub const REVERSE_POLYTONE_NOTE: &str = "eb";
    pub const CONFIG: &str = "ec";
    pub const ACCOUNTS: &str = "ed";
    pub const ACKS: &str = "ee";
}

pub mod ibc_host {
    pub const CHAIN_PROXIES: &str = "fa";
    pub const REVERSE_CHAIN_PROXIES: &str = "fb";
    pub const CONFIG: &str = "fc";
    pub const TEMP_ACTION_AFTER_CREATION: &str = "afd";
}

pub mod ica_client {
    pub const CONFIG: &str = "ga";
}
