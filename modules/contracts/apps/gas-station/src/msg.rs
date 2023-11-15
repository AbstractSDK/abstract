use abstract_core::objects::AnsAsset;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Coin, Timestamp};

use crate::contract::GasStationApp;
use crate::state::GradeName;

abstract_app::app_msg_types!(GasStationApp, GasStationExecuteMsg, GasStationQueryMsg);

/// App instantiate message
/// This message is used to initialize the GasStationApp upon deployment.
#[cosmwasm_schema::cw_serde]
pub struct GasStationInstantiateMsg {
    // Any initialization parameters can be added here if needed in the future.
}

/// Messages that can be executed in the GasStationApp.
/// These messages define the actions that can be taken within the contract.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum GasStationExecuteMsg {
    /// Create a new gas pump with a specific grade and amount.
    /// This allows the contract owner or privileged accounts to define new gas grades.
    CreateGrade {
        /// The grade of the gas (e.g., "premium_100000").
        grade: GradeName,
        /// The amount of gas this grade represents.
        fuel_mix: Vec<AnsAsset>,
        // expiration: Option<Expiration>
    },
    /// Activate a gas pass for a specific recipient.
    ActivateGasPass {
        /// The grade of the gas (e.g., "premium_100000").
        grade: GradeName,
        /// The address to which the gas pass should be issued.
        recipient: String,
        /// When permission should expire
        expiration: Option<Timestamp>,
        /// Bypass the check asserting that the user does not have a gas pass.
        bypass_pass_check: Option<bool>,
        /// Create the account if it does not exist.
        create_if_missing: Option<bool>,
    },
    /// Revoke a gas pass from a specific recipient.
    DeactivateGasPass {
        /// The address to which the gas pass should be issued.
        holder: String,
    },
}

/// Messages that query the state of the GasStationApp.
/// These messages allow users or applications to retrieve information from the contract.
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum GasStationQueryMsg {
    /// Retrieve information about a specific gas pump.
    /// This allows users or applications to understand the details of a specific gas grade.
    #[returns(GradeInfoResponse)]
    GradeInfo {
        grade: GradeName, // The grade of the gas to query.
    },
    /// Retrieve a list of all gas pumps.
    #[returns(GradeListResponse)]
    GradeList {},
    /// Retrieve the holders of a given gas pass
    #[returns(GasPassHoldersResponse)]
    GasPassHolders {
        grade: GradeName, // The grade of the gas to query.
    },
    /// Retrieve the gas pass for a given holder
    #[returns(GasPassResponse)]
    GasPass {
        holder: String, // The holder of the gas pass to query.
    },
}

#[cosmwasm_schema::cw_serde]
pub struct GradeInfoResponse {
    pub grade: GradeName,    // The grade of the gas.
    pub fuel_mix: Vec<Coin>, // The amount of gas this grade represents.
}

#[cosmwasm_schema::cw_serde]
pub struct GradeListResponse {
    pub grades: Vec<GradeInfoResponse>, // The list of gas pumps.
}

#[cosmwasm_schema::cw_serde]
pub struct GasPassHoldersResponse {
    pub holders: Vec<Addr>, // The list of gas pass holders.
}

#[cosmwasm_schema::cw_serde]
pub struct GasPassResponse {
    pub grade: GradeName,              // The grade of the gas.
    pub holder: String,                // The holder of the gas pass.
    pub expiration: Option<Timestamp>, // The expiration of the gas pass.
}
