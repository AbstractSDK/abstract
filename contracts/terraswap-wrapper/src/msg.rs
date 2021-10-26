use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Decimal, Uint128};
use terraswap::asset::Asset;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {
    pub terraswap_pool_addr: String,
    pub trader: String,
    pub max_deposit: Asset,
    pub min_profit: Asset,
    pub slippage: Decimal,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Deposit { funds: Vec<Asset> },
    Withdraw { funds: Vec<Asset> },
    Spend { recipient: String, amount: Asset },
    SetTrader { trader: String },
    SetMaxDeposit { asset: Asset },
    SetMinProfit { asset: Asset },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // Pair {},
    // Pool {},
    WithdrawableProfits {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct WithdrawableProfitsResponse {
    pub amount: Asset,
    pub lp_amount: Uint128,
}
