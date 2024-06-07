//! # Abstract Standalone
//!
//! `abstract_std::standalone` implements shared functionality that's useful for creating new Abstract standalone modules.
//!
//! ## Description
//! An Abstract standalone contract is a contract that is controlled by abstract account, but cannot perform actions on a [proxy](crate::proxy) contract.
use crate::objects::{ans_host::AnsHost, version_control::VersionControlContract};

use cosmwasm_std::Addr;
#[allow(unused_imports)]
use cw_controllers::AdminResponse;
/// Used to instantiate Standalone
#[cosmwasm_schema::cw_serde]
pub struct BaseInstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[cosmwasm_schema::cw_serde]
pub struct StandaloneState {
    pub proxy_address: Addr,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
    /// Used to verify requests
    pub version_control: VersionControlContract,
    /// Used to determine if this standalone is migratable
    pub is_migratable: bool,
}
