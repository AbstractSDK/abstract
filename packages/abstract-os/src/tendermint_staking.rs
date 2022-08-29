//! # Tendermint Staking API
//!
//! `abstract_os::tendermint_staking` exposes all the function of [`cosmwasm_std::CosmosMsg::Staking`] and [`cosmwasm_std::CosmosMsg::Distribution`].

use cosmwasm_std::Uint128;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RequestMsg {
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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct MigrateMsg {}

/// Staking queries are available on [`cosmwasm_std::QuerierWrapper`] through [`cosmwasm_std::Deps`]. Helper function are exposed by [`abstract_sdk::tendermint_staking`]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {}
