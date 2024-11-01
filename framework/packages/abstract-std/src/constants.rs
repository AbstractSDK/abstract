#![allow(missing_docs)]
//! # Registry constants
//!
//! `abstract_std` contains all contract names supported by Abstract.
//!
//! ## Description
//! These fixed names can be used to provide name-addressable searches for complex logic.

pub const ACCOUNT: &str = "abstract:account";
pub const REGISTRY: &str = "abstract:registry";
pub const MODULE_FACTORY: &str = "abstract:module-factory";
pub const ANS_HOST: &str = "abstract:ans-host";
pub const IBC_CLIENT: &str = "abstract:ibc-client";
pub const ICA_CLIENT: &str = "abstract:ica-client";
pub const IBC_HOST: &str = "abstract:ibc-host";

pub const ABSTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

/// IBC protocols
pub const ICS20: &str = "ics-20";

//  ---------------------------
//  Cosmos chains
//  https://cosmos.directory/
//  ---------------------------
pub mod juno {
    pub const JUNO_MAINNET: &str = "juno";
    pub const JUNO_TESTNET: &str = "uni";
    pub const JUNO: &[&str] = &[JUNO_MAINNET, JUNO_TESTNET];
}

pub mod osmosis {
    pub const OSMOSIS_MAINNET: &str = "osmosis";
    pub const OSMOSIS_TESTNET: &str = "osmo-test";
    pub const OSMOSIS: &[&str] = &[OSMOSIS_MAINNET, OSMOSIS_TESTNET];
}

pub mod terra {
    pub const TERRA_MAINNET: &str = "phoenix";
    pub const TERRA_TESTNET: &str = "pisco";
    pub const TERRA: &[&str] = &[TERRA_MAINNET, TERRA_TESTNET];
}

pub mod kujira {
    pub const KUJIRA_MAINNET: &str = "kaiyo";
    pub const KUJIRA_TESTNET: &str = "harpoon";
    pub const KUJIRA: &[&str] = &[KUJIRA_MAINNET, KUJIRA_TESTNET];
}

pub mod neutron {
    pub const NEUTRON_MAINNET: &str = "neutron";
    pub const NEUTRON_TESTNET: &str = "pion";
    pub const NEUTRON: &[&str] = &[NEUTRON_MAINNET, NEUTRON_TESTNET];
}

pub mod archway {
    pub const ARCHWAY_MAINNET: &str = "archway";
    pub const ARCHWAY_TESTNET: &str = "constantine";
    pub const ARCHWAY: &[&str] = &[ARCHWAY_MAINNET, ARCHWAY_TESTNET];
}

pub mod union {
    pub const UNION_TESTNET: &str = "union-testnet";
    pub const UNION: &[&str] = &[UNION_TESTNET];
}

pub mod xion {
    pub const XION_TESTNET: &str = "xion-testnet";
    pub const XION: &[&str] = &[XION_TESTNET];
}

pub mod local {
    pub const MOCK_CHAIN: &str = "cosmos-testnet";
    pub const LOCAL_CHAIN: &[&str] = &[MOCK_CHAIN];
}

pub use archway::ARCHWAY;
pub use juno::JUNO;
pub use kujira::KUJIRA;
pub use local::LOCAL_CHAIN;
pub use neutron::NEUTRON;
pub use osmosis::OSMOSIS;
pub use terra::TERRA;
pub use union::UNION;
pub use xion::XION;

//  ---------------------------
//  EVM chains
//  https://chainlist.org/
//  ---------------------------
pub mod berachain {
    pub const BERACHAIN_BARTIO: &str = "bartio";
    pub const BERACHAIN: &[&str] = &[BERACHAIN_BARTIO];
}

pub mod ethereum {
    pub const ETHEREUM_SEPOLIA: &str = "sepolia";
    pub const ETHEREUM_MAINNET: &str = "ethereum";
    pub const ETHEREUM: &[&str] = &[ETHEREUM_SEPOLIA, ETHEREUM_MAINNET];
}

pub use berachain::BERACHAIN;
pub use ethereum::ETHEREUM;

/// Useful when deploying registry
#[allow(unused)]
pub static NATIVE_CONTRACTS: &[&str] = &[ANS_HOST, MODULE_FACTORY, REGISTRY, "cw20"];
pub static ACCOUNT_CONTRACTS: &[&str] = &[ACCOUNT, ACCOUNT];
pub const ABSTRACT_EVENT_TYPE: &str = "wasm-abstract";

//  ---------------------------
//  Delimiters
//  ---------------------------

/// The delimiter between assets in lists
pub const ASSET_DELIMITER: &str = ",";
/// The delimited between types like contract_type/asset1,asset2
pub const TYPE_DELIMITER: &str = "/";
/// The delimiter between attributes like contract:protocol
pub const ATTRIBUTE_DELIMITER: &str = ":";
/// The delimiter between chains in asset names and traces
/// chain1>chain2>asset
pub const CHAIN_DELIMITER: &str = ">";
