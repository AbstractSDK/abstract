use cosmwasm_std::{Decimal, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::modules::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg, BaseQueryMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoUnchecked};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base: BaseInstantiateMsg,
    pub payment_asset: AssetInfoUnchecked,
    pub subscription_cost: Uint64,
    pub version_control_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    // Add dapp-specific messages here
    Receive(Cw20ReceiveMsg),
    Pay {
        asset: Asset,
        os_id: u32,
    },
    PurgeDebtors{
        page_limit: Option<u32>
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
#[serde(rename_all = "snake_case")]
pub enum DepositHookMsg {
    Pay { os_id: u32 },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub income: Uint64,
    pub next_pay_day: Uint64,
    pub debtors: Vec<u32>,
}
