//! # Tendermint Staking API
//!
//! `abstract_os::tendermint_staking` exposes all the function of [`cosmwasm_std::CosmosMsg::Staking`], [`cosmwasm_std::CosmosMsg::Distribution`] and [`cosmwasm_std::CosmosMsg::StakingQuery`].

use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::api::ApiQueryMsg;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestMsg {
    Delegate {
        validator: String,
        amount: Uint128,
    },
    UndelegateFrom {
        validator: String,
        amount: Option<Uint128>,
    },
    UndelegateAll {},
    Redelegate {
        source_validator: String,
        destination_validator: String,
        amount: Option<Uint128>,
    },
    SetWithdrawAddress {
        /// The new `withdraw_address`
        new_withdraw_address: String,
    },
    WithdrawDelegatorReward {
        /// The `validator_address`
        validator: String,
    },
    WithdrawAllRewards {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

/// Staking queries are available on [`cosmwasm_std::QuerierWrapper`] through [`cosmwasm_std::Deps`]. Helper function are exposed by [`abstract_sdk::tendermint_staking`]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(ApiQueryMsg),
}
