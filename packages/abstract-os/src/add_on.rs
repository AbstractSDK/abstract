//! # Abstract Add-On
//!
//! `abstract_os::add_on` implements shared functionality that's useful for creating new Abstract add-ons.
//!
//! ## Description
//! An add-on is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.

use abstract_ica::IbcResponseMsg;
use cosmwasm_schema::QueryResponses;
use cosmwasm_std::{Addr, Empty};
use cw_controllers::AdminResponse;
use serde::Serialize;

/// Used by Abstract to instantiate the contract
/// The contract is then registered on the version control contract using [`crate::version_control::ExecuteMsg::AddApi`].
#[cosmwasm_schema::cw_serde]
pub struct InstantiateMsg<I: Serialize = Empty> {
    /// base api instantiate information
    pub base: BaseInstantiateMsg,
    /// custom instantiate msg attributes
    pub custom: I,
}

/// Used by Module Factory to instantiate AddOn
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub memory_address: String,
}

/// Interface to the AddOn.
#[cosmwasm_schema::cw_serde]
#[serde(tag = "type")]
pub enum ExecuteMsg<T: Serialize, R: Serialize = Empty> {
    /// An Add-On request.
    Request(T),
    /// A configuration message.
    Configure(BaseExecuteMsg),
    /// IbcReceive to process callbacks
    IbcCallback(IbcResponseMsg),
    /// Receive endpoint for CW20 / external service integrations
    Receive(R),
}

#[cosmwasm_schema::cw_serde]
pub enum BaseExecuteMsg {
    /// Updates the base config
    UpdateConfig { memory_address: Option<String> },
}

#[cosmwasm_schema::cw_serde]
#[serde(tag = "type")]
pub enum QueryMsg<Q: Serialize = Empty> {
    /// An AddOn query message. Forwards the msg to the associated proxy.
    AddOn(Q),
    /// A configuration message to whitelist traders.
    Base(BaseQueryMsg),
}

#[cosmwasm_schema::cw_serde]
#[derive(QueryResponses)]
pub enum BaseQueryMsg {
    /// Returns [`AddOnConfigResponse`]
    #[returns(AddOnConfigResponse)]
    Config {},
    /// Returns the admin.
    #[returns(AdminResponse)]
    Admin {},
}

#[cosmwasm_schema::cw_serde]
pub struct AddOnMigrateMsg {}

#[cosmwasm_schema::cw_serde]
pub struct AddOnConfigResponse {
    pub proxy_address: Addr,
    pub memory_address: Addr,
    pub manager_address: Addr,
}
