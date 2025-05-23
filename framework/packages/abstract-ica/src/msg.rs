use crate::IcaAction;
use abstract_sdk::std::objects::TruncatedChainId;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, CosmosMsg};
use cw_ownable::{cw_ownable_execute, cw_ownable_query};

/// This needs no info. Owner of the contract is whoever signed the InstantiateMsg.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg {
    pub ans_host_address: String,
    pub registry_address: String,
}

#[cosmwasm_schema::cw_serde]
pub enum MigrateMsg {
    Instantiate(InstantiateMsg),
    Migrate {},
}

pub type EvmChainId = u64;

#[cw_ownable_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum ExecuteMsg {
    RegisterInfrastructure {
        /// Chain to register the infrastructure for ("sepolia", "osmosis", "holesky", etc.)
        chain: TruncatedChainId,
        /// Polytone note (locally deployed)
        note: String,
    },
    /// Owner method: Remove connection for remote chain
    RemoveHost { host_chain: TruncatedChainId },
}

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
        // Account address used to query polytone implementations or account itself.
        account_address: String,
        // Chain to send to
        chain: TruncatedChainId,
        // Queries go first
        actions: Vec<IcaAction>,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub ans_host: Addr,
    pub registry_address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct IcaActionResult {
    /// messages that call the underlying implementations (be it polytone/cw-ica-controller/etc)
    pub msgs: Vec<CosmosMsg>,
}
