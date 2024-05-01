//! # Tendermint Staking Adapter
//!
//! `abstract_std::tendermint_staking` exposes all the function of [`cosmwasm_std::CosmosMsg::Staking`] and [`cosmwasm_std::CosmosMsg::Distribution`].

use abstract_adapter::std::adapter;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Empty, Uint128};

pub type InstantiateMsg = adapter::InstantiateMsg<Empty>;
pub type ExecuteMsg = adapter::ExecuteMsg<TendermintStakingExecuteMsg>;
pub type QueryMsg = adapter::QueryMsg<TendermintStakingQueryMsg>;

impl adapter::AdapterExecuteMsg for TendermintStakingExecuteMsg {}
impl adapter::AdapterQueryMsg for TendermintStakingQueryMsg {}

#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
#[impl_into(ExecuteMsg)]
pub enum TendermintStakingExecuteMsg {
    Delegate {
        /// Validator address
        validator: String,
        amount: Uint128,
    },
    UndelegateFrom {
        /// Validator address
        validator: String,
        amount: Option<Uint128>,
    },
    UndelegateAll {},
    Redelegate {
        /// Validator address
        source_validator: String,
        /// Validator address
        destination_validator: String,
        amount: Option<Uint128>,
    },
    SetWithdrawAddress {
        /// The new `withdraw_address`
        new_withdraw_address: String,
    },
    WithdrawDelegatorReward {
        /// Validator address
        validator: String,
    },
    /// Withdraw all the rewards
    WithdrawAllRewards {},
}

/// Staking queries are available on [`cosmwasm_std::QuerierWrapper`] through [`cosmwasm_std::Deps`].
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
#[impl_into(QueryMsg)]
pub enum TendermintStakingQueryMsg {}
