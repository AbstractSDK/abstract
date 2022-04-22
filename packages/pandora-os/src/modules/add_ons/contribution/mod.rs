use cosmwasm_std::{Decimal, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::modules::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg, BaseQueryMsg};
use cw_asset::{Asset, AssetInfo};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base: BaseInstantiateMsg,
    pub ratio: Decimal,
    pub token_cap: Uint128,
    pub mint_price_factor: Decimal,
    pub project_token: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    // Add dapp-specific messages here
    Claim {
        page_limit: Option<u32>,
    },
    UpdateContributor {
        contributor_addr: String,
        compensation: Compensation,
    },
    RemoveContributor {
        contributor_addr: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(BaseQueryMsg),
    // Add dapp-specific queries here
    State {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub total_weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Compensation {
    pub base: u32,
    pub weight: u32,
    pub next_pay_day: Uint64,
    pub expiration: Uint64,
    pub mint_price_factor: Decimal,
}
