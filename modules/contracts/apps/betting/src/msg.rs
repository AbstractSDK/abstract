#![warn(missing_docs)]
//! # Liquidity Interface Add-On
//!
//! `crate::msg` is an app which allows users to deposit into or withdraw from a [`crate::proxy`] contract.
//!
//! ## Description
//! This contract uses the proxy's value calculation configuration to get the value of the assets held in the proxy and the relative value of the deposit asset.
//! It then mints LP tokens that are claimable for an equal portion of the proxy assets at a later date.
//!
//! ---
//! **WARNING:** This mint/burn mechanism can be mis-used by flash-loan attacks if the assets contained are of low-liquidity compared to the etf's size.
//!
//! ## Creation
//! The etf contract can be added on an OS by calling [`ExecuteMsg::InstallModule`](crate::manager::ExecuteMsg::InstallModule) on the manager of the os.
//! ```ignore
//! let etf_init_msg = InstantiateMsg{
//!                deposit_asset: "juno".to_string(),
//!                base: BaseInstantiateMsg{ans_host_address: "juno1...".to_string()},
//!                fee: Decimal::percent(10),
//!                manager_addr: "juno1...".to_string(),
//!                token_code_id: 3,
//!                etf_lp_token_name: Some("demo_etf".to_string()),
//!                etf_lp_token_symbol: Some("DEMO".to_string()),
//!        };
//! let create_module_msg = ExecuteMsg::InstallModule {
//!                 module: Module {
//!                     info: ModuleInfo {
//!                         name: ETF.into(),
//!                         version: None,
//!                     },
//!                     kind: crate::core::modules::ModuleKind::External,
//!                 },
//!                 init_msg: Some(to_binary(&etf_init_msg).unwrap()),
//!        };
//! // Call create_module_msg on manager
//! ```
//!
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` on [`crate::manager`] with `crate::ETF` as module.

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Decimal};
use cw_asset::AssetUnchecked;

use crate::contract::BetApp;
use abstract_core::objects::{AccountId, AssetEntry};
use crate::state::{TrackInfo, TrackId, TrackTeam, NewBet};


abstract_app::app_msg_types!(BetApp, BetExecuteMsg, BetQueryMsg);

/// Init msg
#[cosmwasm_schema::cw_serde]
pub struct BetInstantiateMsg {
    pub rake: Option<Decimal>,
    pub bet_asset: AssetEntry,
}

/// Execute Msg
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum BetExecuteMsg {
    /// CReate a track for the hackathon
    /// Admin only
    CreateTrack(TrackInfo),
    /// Register as a team for the hackathon
    /// Uses the account caller to find the account id
    Register {
        track_id: TrackId,
    },
    /// Register a team for the hackathon
    /// Admin
    UpdateAccounts {
        track_id: TrackId,
        to_add: Vec<AccountId>,
        to_remove: Vec<AccountId>,
    },
    PlaceBets {
        bets: Vec<NewBet>,
    },
    DistributeWinnings {    },
    Withdraw {},
    /// Admin only
    SetWinningTeam {
        track_id: TrackId,
        team_id: AccountId,
    },
    UpdateConfig {
        rake: Option<Decimal>,
    }
}

/// Query Msg
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum BetQueryMsg {
    /// Returns [`TracksResponse`]
    #[returns(TracksResponse)]
    Tracks {
        start_after: Option<TrackId>,
        limit: Option<u32>,
    },
    /// Returns [`OddsResponse`]
    #[returns(OddsResponse)]
    CalculateOdds {
        track_id: TrackId,
        team_id: AccountId,
    },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    // TotalBets {
    //     track_id: TrackId,
    //     team_id: AccountId,
    // }
}

/// Hook when sending CW20 tokens
#[cosmwasm_schema::cw_serde]
pub enum Cw20HookMsg {
    /// Hook for depositing assets
    Deposit {},
    /// Hook for claiming assets for your LP tokens
    Claim {},
}

/// State query response
#[cosmwasm_schema::cw_serde]
pub struct OddsResponse {
    /// Address of the LP token
    pub share_token_address: Addr,
    /// Fee charged on withdrawal
    pub fee: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// Address of the LP token
    pub rake: Decimal,
    pub bet_asset: AssetEntry,
}

#[cosmwasm_schema::cw_serde]
pub struct TrackResponse {
    pub id: TrackId,
    pub name: String,
    pub description: String,
    pub teams: Vec<TrackTeam>,
    pub winning_team: Option<TrackTeam>,
    pub total_bets: u128,
}

#[cosmwasm_schema::cw_serde]
pub struct TracksResponse {
    pub tracks: Vec<TrackResponse>,
}
