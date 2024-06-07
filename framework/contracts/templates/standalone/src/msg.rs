use abstract_standalone::std::{
    ibc::{IbcResponseMsg, ModuleIbcMsg},
    standalone,
};
use cosmwasm_schema::QueryResponses;

/// Standalone instantiate message
#[cosmwasm_schema::cw_serde]
pub struct MyStandaloneInstantiateMsg {
    // This field will get auto-filled by module factory
    pub base: standalone::BaseInstantiateMsg,
    pub count: i32,
}

/// Standalone execute messages
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum MyStandaloneExecuteMsg {
    UpdateConfig {},
    /// Increment count by 1
    Increment {},
    /// Admin method - reset count
    Reset {
        /// Count value after reset
        count: i32,
    },
    // Hooks below are Abstract-specific
    /// IbcReceive to process IBC callbacks
    /// In order to trust this, the apps and adapters verify this comes from the ibc-client contract.
    IbcCallback(IbcResponseMsg),
    /// ModuleIbc endpoint to receive messages from modules on other chains  
    /// In order to trust this, the apps and adapters verify this comes from the ibc-host contract.
    /// They should also trust the sending chain
    ModuleIbc(ModuleIbcMsg),
}

#[cosmwasm_schema::cw_serde]
pub struct MyStandaloneMigrateMsg {}

/// Standalone query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum MyStandaloneQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    #[returns(CountResponse)]
    Count {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}

#[cosmwasm_schema::cw_serde]
pub struct CountResponse {
    pub count: i32,
}
