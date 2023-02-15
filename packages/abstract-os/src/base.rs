use abstract_ica::IbcResponseMsg;
use cosmwasm_std::Empty;

/// EndpointMsg to the base.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg<BaseMsg, AppMsg, ReceiveMsg = Empty> {
    /// A configuration message, defined by the base.
    Base(BaseMsg),
    /// An app request defined by a base consumer.
    App(AppMsg),
    /// IbcReceive to process IBC callbacks
    IbcCallback(IbcResponseMsg),
    /// Receive endpoint for CW20 / external service integrations
    Receive(ReceiveMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<BaseMsg, AppMsg = Empty> {
    /// base instantiate information
    pub base: BaseMsg,
    /// custom instantiate msg
    pub app: AppMsg,
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg<BaseMsg, AppMsg = Empty> {
    /// A query to the base.
    Base(BaseMsg),
    /// Custom query
    App(AppMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg<BaseMsg = Empty, AppMsg = Empty> {
    /// base migrate information
    pub base: BaseMsg,
    /// custom migrate msg
    pub app: AppMsg,
}
