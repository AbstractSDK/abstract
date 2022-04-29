use std::ops::{Add, Sub};

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
    pub subscription: SubscriptionInstantiateMsg,
    pub contribution: ContributionInstantiateMsg,
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionInstantiateMsg {
    pub payment_asset: AssetInfoUnchecked,
    pub subscription_cost: Uint64,
    pub version_control_addr: String,
    pub factory_addr: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContributionInstantiateMsg {
    pub protocol_income_share: Decimal,
    pub emission_user_share: Decimal,
    pub max_emissions_multiple: Decimal,
    pub project_token: String,
    pub emissions_amp_factor: Uint128,
    pub emissions_offset: Uint128,
    pub base_denom: String,
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
    CollectSubs {
        page_limit: Option<u32>,
    },

    ClaimCompensation {
        contributor: Option<String>,
        page_limit: Option<u32>,
    },
    ClaimEmissions {
        os_id: u32,
    },
    UpdateContributor {
        contributor_addr: String,
        compensation: Compensation,
    },
    RemoveContributor {
        contributor_addr: String,
    },
    UpdateSubscriptionConfig{
        payment_asset: Option<AssetInfo>,
        version_control_address: Option<String>,
        factory_address: Option<String>,
        subscription_cost: Option<Uint64>,
    },
    UpdateContributionConfig{
        protocol_income_share: Option<Decimal>,
        emission_user_share: Option<Decimal>,
        max_emissions_multiple: Option<Decimal>,
        project_token: Option<String>,
        emissions_amp_factor: Option<Uint128>,
        emissions_offset: Option<Uint128>,
        base_denom: Option<String>,
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(BaseQueryMsg),
    // Add dapp-specific queries here
    State {},
    Config {},
    Fee {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DepositHookMsg {
    Pay { os_id: u32 },
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionStateResponse {
    pub income: Uint64,
    pub next_pay_day: Uint64,
    pub debtors: Vec<u32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContributionStateResponse {
    pub total_weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub contribution: ContributionStateResponse,
    pub subscription: SubscriptionStateResponse,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionFeeResponse {
    pub fee: Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Compensation {
    pub base: u32,
    pub weight: u32,
    pub next_pay_day: Uint64,
    pub expiration: Uint64,
}

impl Sub for Compensation {
    type Output = (i32, i32);

    fn sub(self, other: Self) -> (i32, i32) {
        (
            self.base as i32 - other.base as i32,
            self.weight as i32 - other.weight as i32,
        )
    }
}
