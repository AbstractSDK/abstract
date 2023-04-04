//! # Registry
//!
//! `abstract_core` contains all contract names supported by Abstract.
//!
//! ## Description
//! These fixed names can be used to provide name-addressable searches for complex logic.

pub const MANAGER: &str = "abstract:manager";
pub const VERSION_CONTROL: &str = "abstract:version-control";
pub const ACCOUNT_FACTORY: &str = "abstract:account-factory";
pub const MODULE_FACTORY: &str = "abstract:module-factory";
pub const PROXY: &str = "abstract:proxy";
pub const ANS_HOST: &str = "abstract:ans-host";
pub const IBC_CLIENT: &str = "abstract:ibc-client";
pub const OSMOSIS_HOST: &str = "abstract:osmosis-host";

/// IBC protocols
pub const ICS20: &str = "ics-20";

/// Useful when deploying version control
#[allow(unused)]
pub static NATIVE_CONTRACTS: &[&str] = &[
    ANS_HOST,
    MODULE_FACTORY,
    ACCOUNT_FACTORY,
    VERSION_CONTROL,
    "cw20",
];
pub static ACCOUNT_CONTRACTS: &[&str] = &[MANAGER, PROXY];
pub const ABSTRACT_EVENT_TYPE: &str = "abstract";
pub const ABSTRACT_EVENT_NAME: &str = "wasm-abstract";
