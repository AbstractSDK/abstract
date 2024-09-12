//! # Abstract Standalone
//!
//! `abstract_std::standalone` implements shared functionality that's useful for creating new Abstract standalone modules.
//!
//! ## Description
//! An Abstract standalone contract is a contract that is controlled by abstract account, but cannot perform actions on a [proxy](crate::proxy) contract.
use crate::{
    objects::{ans_host::AnsHost, version_control::VersionControlContract},
    version_control::Account,
};

/// Data required for the `StandaloneContract::instantiate` function.
#[cosmwasm_schema::cw_serde]
pub struct StandaloneInstantiateMsg {
    pub ans_host_address: String,
    pub version_control_address: String,
}

/// Contains the abstract infrastructure addresses needed the APIs.
#[cosmwasm_schema::cw_serde]
pub struct StandaloneState {
    pub account: Account,
    /// AnsHost contract struct (address)
    pub ans_host: AnsHost,
    /// Used to verify requests
    pub version_control: VersionControlContract,
    /// Used to determine if this standalone is migratable
    pub is_migratable: bool,
}
