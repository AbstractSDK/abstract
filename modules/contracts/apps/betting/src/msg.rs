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
use cosmwasm_std::{Addr, Decimal, Uint128};
use cw_asset::AssetUnchecked;

use crate::contract::BetApp;
use abstract_core::objects::{AccountId, AnsAsset, AssetEntry};
use crate::state::{RoundInfo, RoundId, RoundTeamKey, Bet, AccountOdds, OddsType, RoundStatus};


abstract_app::app_msg_types!(BetApp, BetExecuteMsg, BetQueryMsg);

/// Init msg
#[cosmwasm_schema::cw_serde]
pub struct BetInstantiateMsg {
    pub rake: Option<Decimal>,
}

/// Execute Msg
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum BetExecuteMsg {
    /// Create a round of betting
    /// Admin only
    CreateRound {
        name: String,
        description: String,
        base_bet_token: AssetEntry,
    },
    /// Register as a team for the hackathon
    /// Uses the account caller to find the account id
    Register {
        round_id: RoundId,
    },
    /// Register teams manually for the round, with predefined odds set.
    /// Good for creating games with predefined odds, but payout can exceed account balance.
    UpdateAccounts {
        round_id: RoundId,
        to_add: Vec<AccountOdds>,
        to_remove: Vec<AccountId>,
    },
    /// Place a new bet for a round
    #[cfg_attr(feature = "interface", payable)]
    PlaceBet {
        bet: Bet,
        round_id: RoundId,
    },
    /// Distribute winnings to the winners of the round
    DistributeWinnings {
        round_id: RoundId,
    },
    /// Admin only
    CloseRound {
        round_id: RoundId,
        winner: Option<AccountId>,
    },
    /// Update the config of the contract
    UpdateConfig {
        rake: Option<Decimal>,
    },
}

/// Query Msg
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum BetQueryMsg {
    /// Returns [`RoundResponse`]
    #[returns(RoundResponse)]
    Round {
        round_id: RoundId,
    },
    /// Returns [`RoundsResponse`]
    #[returns(RoundsResponse)]
    ListRounds {
        start_after: Option<RoundId>,
        limit: Option<u32>,
    },
    /// Returns [`OddsResponse`]
    #[returns(OddsResponse)]
    Odds {
        round_id: RoundId,
        team_id: AccountId,
    },
    /// Returns [`ListOddsResponse`]
    #[returns(ListOddsResponse)]
    ListOdds {
        round_id: RoundId,
    },
    /// Returns [`ConfigResponse`]
    #[returns(ConfigResponse)]
    Config {},
    /// Returns [`BetsResponse`]
    #[returns(BetsResponse)]
    Bets {
        round_id: RoundId,
    }
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
    pub round_id: RoundId,
    pub odds: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct ListOddsResponse {
    pub round_id: RoundId,
    pub odds: Vec<AccountOdds>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// Address of the LP token
    pub rake: Decimal,
}

#[cosmwasm_schema::cw_serde]
pub struct BetsResponse {
    pub round_id: RoundId,
    /// Address of the LP token
    pub bets: Vec<(Addr, Uint128)>,
}

#[cosmwasm_schema::cw_serde]
pub struct RoundResponse {
    pub id: RoundId,
    pub name: String,
    pub description: String,
    pub teams: Vec<AccountId>,
    pub status: RoundStatus,
    pub bet_count: u128,
    pub total_bet: AnsAsset
}

#[cosmwasm_schema::cw_serde]
pub struct RoundsResponse {
    pub rounds: Vec<RoundResponse>,
}
