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
pub const ETF: &str = "abstract:etf";
pub const SUBSCRIPTION: &str = "abstract:subscription";
pub const EXCHANGE: &str = "abstract:dex";
pub const TENDERMINT_STAKING: &str = "abstract:tendermint-staking";
pub const CW20_VESTING: &str = "abstract:cw20-vesting";
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
pub static API_CONTRACTS: &[&str] = &[EXCHANGE, TENDERMINT_STAKING];
pub static APPS: &[&str] = &[ETF];
pub static CORE: &[&str] = &[MANAGER, PROXY];

pub const ABSTRACT_EVENT_NAME: &str = "wasm-abstract";
