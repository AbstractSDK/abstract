use abstract_ica::IbcResponseMsg;
use cosmwasm_std::Empty;

/// Wrapper around all possible messages that can be sent to the contract.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg<BaseMsg, ModuleMsg, ReceiveMsg = Empty> {
    /// A configuration message, defined by the base.
    Base(BaseMsg),
    /// An app request defined by a base consumer.
    Module(ModuleMsg),
    /// IbcReceive to process IBC callbacks
    IbcCallback(IbcResponseMsg),
    /// Receive endpoint for CW20 / external service integrations
    Receive(ReceiveMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<BaseMsg, ModuleMsg = Empty> {
    /// base instantiate information
    pub base: BaseMsg,
    /// custom instantiate msg
    pub module: ModuleMsg,
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg<BaseMsg, ModuleMsg = Empty> {
    /// A query to the base.
    Base(BaseMsg),
    /// Custom query
    Module(ModuleMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg<BaseMsg = Empty, ModuleMsg = Empty> {
    /// base migrate information
    pub base: BaseMsg,
    /// custom migrate msg
    pub module: ModuleMsg,
}
