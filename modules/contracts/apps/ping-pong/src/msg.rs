use abstract_app::objects::TruncatedChainId;
use cosmwasm_schema::QueryResponses;

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
    /// Play ping pong between this module and its counterpart on another chain.
    PingPong { opponent_chain: TruncatedChainId },
    /// Same as PingPong but first queries the state of the opponent chain.
    /// If the opponent chain should lose (block height not even), it will try to play.
    QueryAndMaybePingPong { opponent_chain: TruncatedChainId },
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum AppQueryMsg {
    #[returns(GameStatusResponse)]
    GameStatus {},
    /// Returns last ping pong that was initiated through this smart contract
    #[returns(BlockHeightResponse)]
    BlockHeight {},
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
pub enum PingPongCallbackMsg {
    Pinged { opponent_chain: TruncatedChainId },
    QueryBlockHeight { opponent_chain: TruncatedChainId },
}

#[cosmwasm_schema::cw_serde]
pub struct AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct GameStatusResponse {
    pub wins: u32,
    pub losses: u32,
}

#[cosmwasm_schema::cw_serde]
pub struct BlockHeightResponse {
    pub height: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct PreviousPingPongResponse {
    pub pongs: Option<u32>,
    pub host_chain: Option<TruncatedChainId>,
}
