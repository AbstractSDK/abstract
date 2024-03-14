pub const JUNO: &str = "juno-1";
pub const STARGAZE: &str = "stargaze-1";
pub const OSMOSIS: &str = "osmosis-1";

pub mod common;
pub mod interchain_accounts;
pub mod setup;

#[cfg(test)]
pub mod migrate;

#[cfg(test)]
pub mod mars_mm;
