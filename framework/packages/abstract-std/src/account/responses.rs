//! # Account Manager
//!
//! `abstract_std::manager` implements the contract interface and state lay-out.
//!
//! ## Description
//!
//! The Account manager is part of the Core Abstract Account contracts along with the `abstract_std::proxy` contract.
//! This contract is responsible for:
//! - Managing modules instantiation and migrations.
//! - Managing permissions.
//! - Upgrading the Account and its modules.
//! - Providing module name to address resolution.
//!
//! **The manager should be set as the contract/CosmWasm admin by default on your modules.**
//! ## Migration
//! Migrating this contract is done by calling `ExecuteMsg::Upgrade` with `abstract::manager` as module.

use cosmwasm_std::Addr;
use cw2::ContractVersion;

use super::state::AccountInfo;
use crate::{account::state::SuspensionStatus, objects::account::AccountId};

/// Callback message to set the dependencies after module upgrades.
#[cosmwasm_schema::cw_serde]
pub struct CallbackMsg {}

#[cosmwasm_schema::cw_serde]
pub struct ModuleVersionsResponse {
    pub versions: Vec<ContractVersion>,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleAddressesResponse {
    pub modules: Vec<(String, Addr)>,
}

#[cosmwasm_schema::cw_serde]
pub struct InfoResponse {
    pub info: AccountInfo,
}

#[cosmwasm_schema::cw_serde]
pub struct ManagerModuleInfo {
    pub id: String,
    pub version: ContractVersion,
    pub address: Addr,
}

#[cosmwasm_schema::cw_serde]
pub struct ModuleInfosResponse {
    pub module_infos: Vec<ManagerModuleInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct SubAccountIdsResponse {
    pub sub_accounts: Vec<u32>,
}

#[cosmwasm_schema::cw_serde]
pub struct ConfigResponse {
    pub modules: Vec<String>,
    pub account_id: AccountId,
    pub is_suspended: SuspensionStatus,
    pub version_control_address: Addr,
    pub module_factory_address: Addr,
}
