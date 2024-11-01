//! Defines the messages generics used for all types of Abstract Modules
//!
//! Those types define a common interface for all Abstract Modules

use cosmwasm_schema::QueryResponses;
use cosmwasm_std::Empty;

use crate::ibc::{IbcResponseMsg, ModuleIbcMsg};

// ANCHOR: exec
/// Wrapper around all possible execution messages that can be sent to the module.
#[cosmwasm_schema::cw_serde]
pub enum ExecuteMsg<BaseMsg, CustomExecMsg> {
    /// A configuration message, defined by the base.
    Base(BaseMsg),
    /// An app request defined by a base consumer.
    Module(CustomExecMsg),
    /// IbcReceive to process IBC callbacks
    /// In order to trust this, the apps and adapters verify this comes from the ibc-client contract.
    IbcCallback(IbcResponseMsg),
    /// ModuleIbc endpoint to receive messages from modules on other chains  
    /// In order to trust this, the apps and adapters verify this comes from the ibc-host contract.
    /// They should also trust the sending chain
    ModuleIbc(ModuleIbcMsg),
}
// ANCHOR_END: exec

// ANCHOR: init
/// InstantiateMsg for modules.
///
/// Contains a base part needed to init Abstract related storage.
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<BaseMsg, CustomInitMsg = Empty> {
    /// base instantiate information
    pub base: BaseMsg,
    /// custom instantiate msg
    pub module: CustomInitMsg,
}
// ANCHOR_END: init

// ANCHOR: query
/// Wrapper around all possible queries that can be sent to the module.
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
/// MigrateMsg for modules.
///
/// Contains a base part needed to init Abstract related storage.
#[cosmwasm_schema::cw_serde]
pub struct MigrateMsg<BaseMsg = Empty, CustomMigrateMsg = Empty> {
    /// base migrate information
    pub base: BaseMsg,
    /// custom migrate msg
    pub module: CustomMigrateMsg,
}
// ANCHOR_END: migrate
