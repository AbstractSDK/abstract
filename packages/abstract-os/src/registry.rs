//! # Registry
//!
//! `abstract_os` contains all contract names supported by Abstract OS.
//!
//! ## Description
//! These fixed names can be used to provide name-addressable searches for complex logic.

pub const MANAGER: &str = "abstract:manager";
pub const VERSION_CONTROL: &str = "abstract:version-control";
pub const OS_FACTORY: &str = "abstract:os-factory";
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
    OS_FACTORY,
    VERSION_CONTROL,
    "cw20",
];
pub static CORE: &[&str] = &[MANAGER, PROXY];
pub const ABSTRACT_EVENT_NAME: &str = "wasm-abstract";
