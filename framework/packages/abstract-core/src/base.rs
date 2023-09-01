use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Empty;

use crate::ibc::IbcResponseMsg;

// ANCHOR: exec
/// Wrapper around all possible messages that can be sent to the module.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg<BaseMsg, CustomExecMsg, ReceiveMsg = Empty> {
    /// A configuration message, defined by the base.
    Base(BaseMsg),
    /// An app request defined by a base consumer.
    Module(CustomExecMsg),
    /// IbcReceive to process IBC callbacks
    /// In order to trust this, they should verify this comes from the a note contract.
    /// This needs to be verified inside apps
    /// TODO, do we need to name this variant IbcCallBack or Callback (Polytone calls it callback)
    IbcCallback(IbcResponseMsg),
    /// Receive endpoint for CW20 / external service integrations
    Receive(ReceiveMsg),
}
// ANCHOR_END: exec

// ANCHOR: init
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<BaseMsg, CustomInitMsg = Empty> {
    /// base instantiate information
    pub base: BaseMsg,
    /// custom instantiate msg
    pub module: CustomInitMsg,
}
// ANCHOR_END: init

// ANCHOR: query
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
#[query_responses(nested)]
pub enum QueryMsg<BaseMsg, CustomQueryMsg = Empty> {
    /// A query to the base.
    Base(BaseMsg),
    /// Custom query
    Module(CustomQueryMsg),
}
// ANCHOR_END: query

// ANCHOR: migrate
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg<BaseMsg = Empty, CustomMigrateMsg = Empty> {
    /// base migrate information
    pub base: BaseMsg,
    /// custom migrate msg
    pub module: CustomMigrateMsg,
}
// ANCHOR_END: migrate
