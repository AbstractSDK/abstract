use abstract_core::{app, objects::AccountId};
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Decimal, Timestamp, Uint128};
use cw_asset::AssetInfoUnchecked;

use super::state::{Compensation, ContributionState, ContributorsConfig};

/// Top-level Abstract App execute message. This is the message that is passed to the `execute` entrypoint of the smart-contract.
pub type ExecuteMsg = app::ExecuteMsg<AppExecuteMsg>;
/// Top-level Abstract App instantiate message. This is the message that is passed to the `instantiate` entrypoint of the smart-contract.
pub type InstantiateMsg = app::InstantiateMsg<ContributorsInstantiateMsg>;
/// Top-level Abstract App query message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type QueryMsg = app::QueryMsg<AppQueryMsg>;
/// Top-level Abstract App migrate message. This is the message that is passed to the `query` entrypoint of the smart-contract.
pub type MigrateMsg = app::MigrateMsg<AppMigrateMsg>;

impl app::AppExecuteMsg for AppExecuteMsg {}
impl app::AppQueryMsg for AppQueryMsg {}

/// Contributors instantiation message
#[cosmwasm_schema::cw_serde]
pub struct ContributorsInstantiateMsg {
    /// Percentage of income that is redirected to the protocol
    pub protocol_income_share: Decimal,
    /// Percentage of emissions allocated to users
    pub emission_user_share: Decimal,
    /// Max emissions (when income = 0) = max_emissions_multiple * floor_emissions
    pub max_emissions_multiple: Decimal,
    /// Emissions amplification factor in inverse emissions <-> target equation
    pub emissions_amp_factor: Uint128,
    /// Emissions offset factor in inverse emissions <-> target equation
    pub emissions_offset: Uint128,
    /// token: TODO
    pub token_info: AssetInfoUnchecked,
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum AppExecuteMsg {
    /// Update config of contributors
    UpdateConfig {
        /// New ercentage of income that is redirected to the protocol
        protocol_income_share: Option<Decimal>,
        /// New ercentage of emissions allocated to users
        emission_user_share: Option<Decimal>,
        /// New max emissions (when income = 0) = max_emissions_multiple * floor_emissions
        max_emissions_multiple: Option<Decimal>,
        /// New emissions amplification factor in inverse emissions <-> target equation
        emissions_amp_factor: Option<Uint128>,
        /// New emissions offset factor in inverse emissions <-> target equation
        emissions_offset: Option<Uint128>,
        /// Change project token
        project_token_info: Option<AssetInfoUnchecked>,
    },
    /// Claim the compensation for contributor
    ClaimCompensation {
        /// Abstract account id of contributor
        os_id: AccountId,
    },
    /// Update/add the contributor config
    UpdateContributor {
        /// Abstract account id of contributor
        os_id: AccountId,
        /// Base amount payment per week
        base_per_week: Option<Decimal>,
        /// Weight of the contributor
        weight: Option<u32>,
        /// Timestamp when "contract" with this contributor expires
        expiration_timestamp: Option<Timestamp>,
    },
    /// Remove the contributor
    RemoveContributor {
        /// Abstract account id of contributor
        os_id: AccountId,
    },
}

/// App query messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum AppQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(StateResponse)]
    State {},
    /// Get state of the contributor
    #[returns(ContributorStateResponse)]
    ContributorState {
        /// Abstract Account Id of contributor
        os_id: AccountId,
    },
}

#[cosmwasm_schema::cw_serde]
pub enum AppMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// Config for the contributors
    pub config: ContributorsConfig,
}

#[cosmwasm_schema::cw_serde]
pub struct StateResponse {
    /// State of contributors
    pub contribution: ContributionState,
}

/// Query response for [`SubscriptionQueryMsg::ContributorState`]
#[cosmwasm_schema::cw_serde]
pub struct ContributorStateResponse {
    /// Compensation details for contributors
    pub compensation: Compensation,
}

#[cosmwasm_schema::cw_serde]
pub enum ContributorsExecuteMsg {}
