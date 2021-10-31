use cosmwasm_std::{Addr, Uint128};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Spend { recipient: String, amount: Uint128 },
    Burn { amount: Uint128 },
    Deposit {},
    UpdateAdmin { admin: String },
    UpdateAnchorDepositThreshold { threshold: Uint128 },
    UpdateAnchorWithdrawThreshold { threshold: Uint128 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Admin {},
    Config {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub token_addr: Addr,
    pub ust_pool_addr: Addr,
    pub anchor_money_market_addr: Addr,
    pub aust_addr: Addr,
    pub anchor_deposit_threshold: Uint128,
    pub anchor_withdraw_threshold: Uint128,
}
