use std::fmt::Display;

use abstract_sdk::std::objects::TruncatedChainId;
use abstract_sdk::std::registry::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ChainType {
    Evm,
    Cosmos,
}

impl Display for ChainType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ChainType::Evm => write!(f, "EVM"),
            ChainType::Cosmos => write!(f, "Cosmos"),
        }
    }
}

pub trait CastChainType {
    fn chain_type(&self) -> Option<ChainType>;
}

impl CastChainType for TruncatedChainId {
    // Return the type of chain based on the chain-id.
    // Note: chain-ids for EVM chains are numbers!
    fn chain_type(&self) -> Option<ChainType> {
        // TODO: Should be a better way to have lookup table, maybe this will be nice:
        // https://crates.io/crates/phf
        let chains = map_macro::hash_map! {
            ARCHWAY[0] => ChainType::Cosmos,
            ARCHWAY[1] => ChainType::Cosmos,
            NEUTRON[0] => ChainType::Cosmos,
            NEUTRON[1] => ChainType::Cosmos,
            KUJIRA[0] => ChainType::Cosmos,
            KUJIRA[1] => ChainType::Cosmos,
            TERRA[0] => ChainType::Cosmos,
            TERRA[1] => ChainType::Cosmos,
            OSMOSIS[0] => ChainType::Cosmos,
            OSMOSIS[1] => ChainType::Cosmos,
            JUNO[0] => ChainType::Cosmos,
            JUNO[1] => ChainType::Cosmos,

            // Only Testnet
            UNION[0] => ChainType::Cosmos,
            XION[0] => ChainType::Cosmos,

            // EVM
            BERACHAIN[0] => ChainType::Evm,
            ETHEREUM[0] => ChainType::Evm,
            ETHEREUM[1] => ChainType::Evm,
        };

        chains.get(self.as_str()).copied()
    }
}
