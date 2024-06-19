use abstract_standalone::std::standalone;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::CosmosMsg;
use cw_ica_controller::{
    helpers::ica_callback_execute, types::msg::options::ChannelOpenInitOptions,
};

/// Standalone instantiate message
#[cosmwasm_schema::cw_serde]
pub struct MyStandaloneInstantiateMsg {
    // This field will get auto-filled by module factory
    pub base: standalone::StandaloneInstantiateMsg,
    /// The code ID of the cw-ica-controller contract.
    pub ica_controller_code_id: u64,
}

/// Standalone execute messages
#[ica_callback_execute]
#[cosmwasm_schema::cw_serde]
#[derive(cw_orch::ExecuteFns)]
pub enum MyStandaloneExecuteMsg {
    CreateIcaContract {
        salt: Option<String>,
        channel_open_init_options: ChannelOpenInitOptions,
    },
    SendAction {
        /// The ICA ID.
        ica_id: u64,
        /// Message to the ICA
        msg: CosmosMsg,
    },
}

#[cosmwasm_schema::cw_serde]
pub struct MyStandaloneMigrateMsg {}

/// Standalone query messages
#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses, cw_orch::QueryFns)]
pub enum MyStandaloneQueryMsg {
    #[returns(ConfigResponse)]
    Config {},
    /// IcaState returns the ICA state for the given ICA ID.
    #[returns(crate::state::IcaContractState)]
    IcaContractState { ica_id: u64 },
    #[returns(ICACountResponse)]
    ICACount {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {}

#[cosmwasm_schema::cw_serde]
pub struct ICACountResponse {
    pub count: u64,
}
