//! # Registry
//!
//! `abstract_os` contains all contract names supported by Abstract OS.
//!
//! ## Description
//! These fixed names can be used to provide name-addressable searches for complex logic.

pub const MANAGER: &str = "abstract:manager";
pub const VERSION_CONTROL: &str = "abstract:version_control";
pub const OS_FACTORY: &str = "abstract:os_factory";
pub const MODULE_FACTORY: &str = "abstract:module_factory";
pub const PROXY: &str = "abstract:proxy";
pub const MEMORY: &str = "abstract:memory";
pub const ETF: &str = "abstract:etf";
pub const SUBSCRIPTION: &str = "abstract:subscription";
pub const EXCHANGE: &str = "abstract:dex";
pub const TENDERMINT_STAKING: &str = "abstract:tendermint_staking";
pub const CW20_VESTING: &str = "abstract:cw20_vesting";
pub const IBC_CLIENT: &str = "abstract:ibc_client";
pub const OSMOSIS_HOST: &str = "abstract:osmosis_host";

/// IBC protocols
pub const ICS20: &str = "ics-20";

/// Useful when deploying version control
#[allow(unused)]
pub static NATIVE_CONTRACTS: &[&str] =
    &[MEMORY, MODULE_FACTORY, OS_FACTORY, VERSION_CONTROL, "cw20"];
pub static API_CONTRACTS: &[&str] = &[EXCHANGE, TENDERMINT_STAKING];
pub static APPS: &[&str] = &[ETF];
pub static CORE: &[&str] = &[MANAGER, PROXY];
