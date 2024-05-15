//! # Registry
//!
//! `abstract_std` contains all contract names supported by Abstract.
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
pub const IBC_HOST: &str = "abstract:ibc-host";

pub const PROFILE: &str = "bitsong:profile";
pub const PROFILE_MARKETPLACE: &str = "bitsong:profile-marketplace";

/// IBC protocols
pub const ICS20: &str = "ics-20";

// chain-id prefixes based on `https://cosmos.directory/`
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
pub const ABSTRACT_EVENT_TYPE: &str = "wasm-abstract";
