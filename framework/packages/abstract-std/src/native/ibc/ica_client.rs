use crate::objects::TruncatedChainId;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

pub use action::{IcaAction, IcaActionResponse, IcaExecute};
pub use chain_type::{CastChainType, ChainType};

pub use polytone_evm::EVM_NOTE_ID;
pub use polytone_evm::POLYTONE_EVM_VERSION;

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {}

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg {
    Instantiate(InstantiateMsg),
    Migrate {},
}

#[cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {}

#[cw_ownable_query]
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum QueryMsg {
    /// Returns config
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},

    #[returns(IcaActionResult)]
    IcaAction {
        // Account address address used to query polytone implementations or proxy itself.
        account_address: String,
        // Chain to send to
        chain: TruncatedChainId,
        // Queries go first
        actions: Vec<IcaAction>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host_address: Addr,
    pub registry_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct IcaActionResult {
    /// messages that call the underlying implementations (be it polytone/cw-ica-controller/etc)
    pub msgs: Vec<CosmosMsg>,
}

mod chain_type {
    use std::fmt::Display;

    use crate::constants::*;
    use crate::objects::TruncatedChainId;

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
}

mod action {
    use cosmwasm_std::{Binary, Coin, CosmosMsg};

    /// Interchain Account Action
    #[cosmwasm_schema::cw_serde]
    #[non_exhaustive]
    pub enum IcaAction {
        // Execute on the ICA
        Execute(IcaExecute),
        // Send funds to the ICA
        Fund {
            funds: Vec<Coin>,
            // Optional receiver address
            // Should be formatted in expected formatting
            // EVM: HexBinary
            // Cosmos: Addr
            receiver: Option<Binary>,
            memo: Option<String>,
        },
    }

    #[cosmwasm_schema::cw_serde]
    #[non_exhaustive]
    pub enum IcaExecute {
        Evm {
            msgs: Vec<polytone_evm::evm::EvmMsg<String>>,
            callback: Option<polytone_evm::callbacks::CallbackRequest>,
        },
    }

    #[cosmwasm_schema::cw_serde]
    pub struct IcaActionResponse {
        /// messages that call the underlying implementations (be it polytone/cw-ica-controller/etc)
        pub msgs: Vec<CosmosMsg>,
    }
}
