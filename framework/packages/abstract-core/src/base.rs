use abstract_ica::IbcResponseMsg;
use cosmwasm_std::Empty;

// ANCHOR: exec
/// Wrapper around all possible messages that can be sent to the module.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg<BaseMsg, CustomExecMsg, ReceiveMsg = Empty> {
    /// A configuration message, defined by the base.
    Base(BaseMsg),
    /// An app request defined by a base consumer.
    Module(CustomExecMsg),
    /// IbcReceive to process IBC callbacks
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
