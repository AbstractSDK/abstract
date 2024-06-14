use abstract_app::objects::{chain_name::ChainName, AccountId};
use cosmwasm_schema::QueryResponses;
use either::Either;

use crate::contract::App;

// This is used for type safety and re-exporting the contract endpoint structs.
abstract_app::app_msg_types!(App, AppExecuteMsg, AppQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct AppInstantiateMsg {}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum AppExecuteMsg {
    /// PingPong between this module on other chain
    PingPong {
        /// Host chain
        host_chain: ChainName,
    },
    /// Rematch ping pong if host chain ping ponged us
    Rematch {
        host_chain: ChainName,
        account_id: AccountId,
    },
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum AppQueryMsg {
    #[returns(PongsResponse)]
    Pongs {},
    /// Returns last ping pong that was initiated through this smart contract
    #[returns(PreviousPingPongResponse)]
    PreviousPingPong {},
}

#[cosmwasm_schema::cw_serde]
pub enum PingOrPong {
    Ping,
    Pong,
}

#[cosmwasm_schema::cw_serde]
pub struct PingPongIbcMsg {
    pub hand: PingOrPong,
}

#[cosmwasm_schema::cw_serde]
pub struct AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct PongsResponse {
    pub pongs: u32,
}

#[cosmwasm_schema::cw_serde]
pub struct PreviousPingPongResponse {
    pub pongs: Option<u32>,
    pub host_chain: Option<ChainName>,
}
