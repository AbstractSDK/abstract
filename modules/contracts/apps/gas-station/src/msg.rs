use abstract_core::objects::AnsAsset;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Coin;
use cw_asset::Asset;

use crate::contract::GasStationApp;

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
    CreateGasPump {
        grade: String,           // The grade of the gas (e.g., "premium_100000").
        fuel_mix: Vec<AnsAsset>, // The amount of gas this grade represents.
    },
    /// Dispense gas to a specific recipient.
    /// This allows users or applications to request gas tokens of a specific grade.
    DispenseGas {
        grade: String,     // The grade of the gas.
        recipient: String, // The address to which the gas token should be issued.
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
    #[returns(GasPumpInfoResponse)]
    GasPumpInfo {
        grade: String, // The grade of the gas to query.
    },
    /// Retrieve a list of all gas pumps.
    #[returns(GasPumpListResponse)]
    GasPumpList {}
}

/// SudoMsg is only exposed for internal Cosmos SDK modules to call.
/// This is showing how we can expose "admin" functionality than can not be called by
/// external users or contracts, but only trusted (native/Go) code in the blockchain
#[cosmwasm_schema::cw_serde]
pub enum GasStationSudoMsg {
    BlockBeforeSend {
        from: String,
        to: String,
        amount: Coin,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct GasPumpInfoResponse {
    pub grade: String,        // The grade of the gas.
    pub denom: String,        // The denom of the gas pump token.
    pub fuel_mix: Vec<Asset>, // The amount of gas this grade represents.
}

#[cosmwasm_schema::cw_serde]
pub struct GasPumpListResponse {
    pub pumps: Vec<GasPumpInfoResponse>, // The list of gas pumps.
}
