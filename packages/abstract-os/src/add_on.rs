//! # Abstract Add-On Base
//!
//! `abstract_os::add_on` implements shared functionality that's useful for creating new Abstract add-ons.
//!
//! ## Description
//! An add-on is a contract that is allowed to perform actions on a [proxy](crate::proxy) contract while also being migratable.
//! The source code is accessible on [todo](todo).

use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// Used by Module Factory to instantiate AddOn
pub struct AddOnInstantiateMsg {
    pub memory_address: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
#[serde(rename_all = "snake_case")]
pub enum AddOnExecuteMsg {
    /// Updates the base config
    UpdateConfig { memory_address: Option<String> },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AddOnQueryMsg {
    /// Returns [`AddOnConfigResponse`]
    Config {},
    /// Returns the admin.
    Admin {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct AddOnMigrateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct AddOnConfigResponse {
    pub proxy_address: Addr,
    pub memory_address: Addr,
    pub manager_address: Addr,
}

pub mod state {}
