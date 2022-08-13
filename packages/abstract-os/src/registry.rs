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
pub const LIQUIDITY_INTERFACE: &str = "abstract:liquidity_interface";
pub const SUBSCRIPTION: &str = "abstract:subscription";
pub const EXCHANGE: &str = "abstract:dex";
pub const TENDERMINT_STAKING: &str = "abstract:tendermint_staking";

/// Useful when deploying version control
#[allow(unused)]
pub static NATIVE_CONTRACTS: &[&str] =
    &[MEMORY, MODULE_FACTORY, OS_FACTORY, VERSION_CONTROL, "cw20"];
pub static API_CONTRACTS: &[&str] = &[EXCHANGE, TENDERMINT_STAKING];
