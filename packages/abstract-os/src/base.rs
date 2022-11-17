use abstract_ica::IbcResponseMsg;
use cosmwasm_std::Empty;

/// EndpointMsg to the Middleware.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg<BaseMsg, AppMsg, ReceiveMsg = Empty> {
    /// A configuration message.
    Base(BaseMsg),
    /// An app request.
    App(AppMsg),
    /// IbcReceive to process callbacks
    IbcCallback(IbcResponseMsg),
    /// Receive endpoint for CW20 / external service integrations
    Receive(ReceiveMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<BaseMsg, AppMsg = Empty> {
    /// base instantiate information
    pub base: BaseMsg,
    /// custom instantiate msg attributes
    pub app: AppMsg,
}

#[cosmwasm_schema::cw_serde]
pub enum QueryMsg<BaseMsg, AppMsg = Empty> {
    /// A configuration message to the base.
    Base(BaseMsg),
    /// Custom query
    App(AppMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg<BaseMsg = Empty, AppMsg = Empty> {
    /// base migrate information
    pub base: BaseMsg,
    /// custom migrate msg attributes
    pub app: AppMsg,
}
