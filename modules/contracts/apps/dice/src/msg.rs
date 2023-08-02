use cosmwasm_schema::QueryResponses;

use crate::contract::DiceApp;

abstract_app::app_msg_types!(DiceApp, DiceExecuteMsg, DiceQueryMsg);

/// App instantiate message
#[cosmwasm_schema::cw_serde]
pub struct DiceAppInstantiateMsg {
}

/// App execute messages
#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::ExecuteFns))]
#[cfg_attr(feature = "interface", impl_into(ExecuteMsg))]
pub enum DiceExecuteMsg {
    // job_id for this job which allows for gathering the results.
    RollDice { job_id: String }
}

#[cosmwasm_schema::cw_serde]
#[cfg_attr(feature = "interface", derive(cw_orch::QueryFns))]
#[cfg_attr(feature = "interface", impl_into(QueryMsg))]
#[derive(QueryResponses)]
pub enum DiceQueryMsg {
    // GetCount returns the current count as a json-encoded number
    #[returns(String)]
    QueryOutcome { job_id: String },
    #[returns(Vec<String>)]
    GetHistoryOfRounds {},
}
