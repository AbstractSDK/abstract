use cosmwasm_std::Decimal;
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::{
    modules::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg, BaseQueryMsg},
    util::fee::Fee,
};
use cw_asset::AssetUnchecked;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base: BaseInstantiateMsg,
    pub token_code_id: u64,
    pub fee: Decimal,
    pub provider_addr: String,
    pub deposit_asset: String,
    pub vault_lp_token_name: Option<String>,
    pub vault_lp_token_symbol: Option<String>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Base(BaseExecuteMsg),
    // Add dapp-specific messages here
    Receive(Cw20ReceiveMsg),
    ProvideLiquidity {
        asset: AssetUnchecked,
    },
    UpdatePool {
        deposit_asset: Option<String>,
        assets_to_add: Vec<String>,
        assets_to_remove: Vec<String>,
    },
    SetFee {
        fee: Fee,
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
    WithdrawLiquidity {},
    ProvideLiquidity {},
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub liquidity_token: String,
}
