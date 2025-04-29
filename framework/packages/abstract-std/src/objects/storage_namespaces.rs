/// namespace key for State Item
pub const BASE_STATE: &str = "base_state";
/// namespace for contract Admin
pub const ADMIN_NAMESPACE: &str = "admin";
/// storage key for cw_ownable::Ownership
pub const OWNERSHIP_STORAGE_KEY: &str = "ownership";
/// storage key for ModuleData
pub const MODULE_STORAGE_KEY: &str = "mod";
/// Storage key for config in all modules
pub const CONFIG_STORAGE_KEY: &str = "cfg";

pub mod account {
    pub const SUSPENSION_STATUS: &str = "aa";
    pub const INFO: &str = "ab";
    pub const ACCOUNT_MODULES: &str = "ac";
    pub const DEPENDENTS: &str = "ad";
    pub const SUB_ACCOUNTS: &str = "ae";
    pub const WHITELISTED_MODULES: &str = "af";
    pub const ACCOUNT_ID: &str = "ag";
    pub const INSTALL_MODULES_CONTEXT: &str = "ah";
    pub const MIGRATE_CONTEXT: &str = "ai";
    pub const CALLING_TO_AS_ADMIN: &str = "aj";

    // XION authentificators, could be there could be not
    #[cfg(feature = "xion")]
    pub const AUTH_ADMIN: &str = "ax";
}

pub mod ans_host {
    pub const ASSET_ADDRESSES: &str = "ba";
    pub const REV_ASSET_ADDRESSES: &str = "bb";
    pub const CONTRACT_ADDRESSES: &str = "bc";
    pub const CHANNELS: &str = "bd";
    pub const REGISTERED_DEXES: &str = "be";
    pub const ASSET_PAIRINGS: &str = "bf";
    pub const POOL_METADATA: &str = "bg";
}

pub mod registry {
    pub const PENDING_MODULES: &str = "ca";
    pub const REGISTERED_MODULES: &str = "cb";
    pub const STANDALONE_INFOS: &str = "cc";
    pub const SERVICE_INFOS: &str = "cd";
    pub const YANKED_MODULES: &str = "ce";
    pub const MODULE_CONFIG: &str = "cf";
    pub const MODULE_DEFAULT_CONFIG: &str = "cg";
    pub const ACCOUNT_ADDRESSES: &str = "ch";
    pub const LOCAL_ACCOUNT_SEQUENCE: &str = "ci";
    pub const NAMESPACES: &str = "cj";
    pub const REV_NAMESPACES: &str = "ck";
}

pub mod module_factory {
    pub const CURRENT_BASE: &str = "da";
}
pub mod ibc_client {
    pub const IBC_INFRA: &str = "ea";
    pub const REVERSE_POLYTONE_NOTE: &str = "eb";
    pub const ACCOUNTS: &str = "ec";
    pub const ACKS: &str = "ed";
    pub const ICS20_ACCOUNT_CALLBACKS: &str = "ee";
    pub const ICS20_ACCOUNT_CALLBACK_PAYLOAD: &str = "ef";
}

pub mod ibc_host {
    pub const CHAIN_PROXIES: &str = "fa";
    pub const REVERSE_CHAIN_PROXIES: &str = "fb";
    pub const TEMP_ACTION_AFTER_CREATION: &str = "fc";
}

pub mod ica_client {}
