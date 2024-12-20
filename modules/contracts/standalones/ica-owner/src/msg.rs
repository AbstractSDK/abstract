use abstract_standalone::std::standalone;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{CosmosMsg, QueryRequest};
use cw_ica_controller::{
    helpers::ica_callback_execute, types::msg::options::ChannelOpenInitOptions,
};
use cw_ica_controller::ibc::types::packet::acknowledgement::Data;
use cw_ica_controller::types::query_msg::IcaQueryResult;

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
    #[cw_orch(fn_name("ica_execute"))]
    Execute {
        /// The ICA ID.
        ica_id: u64,
        /// Message to the ICA
        msgs: Vec<CosmosMsg>,
    },
    #[cw_orch(fn_name("ica_query"))]
    Query {
        /// The ICA ID.
        ica_id: u64,
        /// Message to the ICA
        msgs: Vec<QueryRequest>,
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
    #[returns(PacketStateResponse)]
    PacketState { ica_id: u64, sequence: u64 },
    #[returns(ICACountResponse)]
    ICACount {},
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    /// The code ID of the cw-ica-controller contract.
    pub ica_controller_code_id: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct ICACountResponse {
    pub count: u64,
}

#[cosmwasm_schema::cw_serde]
pub struct PacketStateResponse {
    pub ack_data: Option<Data>,
    pub query_result: Option<IcaQueryResult>,
}