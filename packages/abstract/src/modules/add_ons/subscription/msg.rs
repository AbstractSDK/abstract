use cosmwasm_std::{Decimal, Uint128, Uint64};
use cw20::Cw20ReceiveMsg;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::common_module::add_on_msg::{AddOnExecuteMsg, AddOnInstantiateMsg, AddOnQueryMsg};
use cw_asset::{Asset, AssetInfo, AssetInfoUnchecked};

use super::state::{
    Compensation, ContributionConfig, ContributionState, Subscriber, SubscriptionConfig,
    SubscriptionState,
};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub base: AddOnInstantiateMsg,
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
    Base(AddOnExecuteMsg),
    // Add dapp-specific messages here
    Receive(Cw20ReceiveMsg),
    Pay {
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
    UpdateSubscriptionConfig {
        payment_asset: Option<AssetInfo>,
        version_control_address: Option<String>,
        factory_address: Option<String>,
        subscription_cost: Option<Uint64>,
    },
    UpdateContributionConfig {
        protocol_income_share: Option<Decimal>,
        emission_user_share: Option<Decimal>,
        max_emissions_multiple: Option<Decimal>,
        project_token: Option<String>,
        emissions_amp_factor: Option<Uint128>,
        emissions_offset: Option<Uint128>,
        base_denom: Option<String>,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Base(AddOnQueryMsg),
    // Add dapp-specific queries here
    State {},
    Config {},
    Fee {},
    SubscriberState { os_id: u32 },
    ContributorState { contributor_addr: String },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DepositHookMsg {
    Pay { os_id: u32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ConfigResponse {
    pub contribution: ContributionConfig,
    pub subscription: SubscriptionConfig,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct StateResponse {
    pub contribution: ContributionState,
    pub subscription: SubscriptionState,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriptionFeeResponse {
    pub fee: Asset,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SubscriberStateResponse {
    pub currently_subscribed: bool,
    pub subscriber_details: Subscriber,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct ContributorStateResponse {
    pub compensation: Compensation,
}
