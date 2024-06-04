//! # Abstract Standalone
//!
//! `abstract_std::standalone` implements shared functionality that's useful for creating new Abstract standalone modules.
//!
//! ## Description
//! TODO:
use crate::objects::{ans_host::AnsHost, version_control::VersionControlContract};

use cosmwasm_std::Addr;
#[allow(unused_imports)]
use cw_controllers::AdminResponse;
/// Used by Module Factory to instantiate Standalone
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
}

#[cosmwasm_schema::cw_serde]
pub struct AppConfigResponse {
    pub ans_host_address: Addr,
    pub manager_address: Addr,
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[cosmwasm_schema::cw_serde]
pub struct StandaloneState {
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
    /// Used to verify requests
    pub version_control: VersionControlContract,
}
