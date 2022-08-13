use std::fmt;

use cosmwasm_std::{to_binary, Binary, StdError, StdResult};
use cw2::ContractVersion;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ModuleInfo {
    pub name: String,
    pub version: Option<String>,
}

impl fmt::Display for ModuleInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Name: {}, Version: {})",
            self.name,
            self.version
                .clone()
                .unwrap_or_else(|| String::from("undefined"))
        )
    }
}

impl From<ContractVersion> for ModuleInfo {
    fn from(contract: ContractVersion) -> Self {
        ModuleInfo {
            name: contract.contract,
            version: Some(contract.version),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Module {
    pub info: ModuleInfo,
    pub kind: ModuleKind,
}

impl fmt::Display for Module {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Info: [{}], Kind: [{:?}]", self.info, self.kind)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ModuleKind {
    AddOn,
    API,
    Service,
    Perk,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ModuleInitMsg {
    pub fixed_init: Option<Binary>,
    pub root_init: Option<Binary>,
}

impl ModuleInitMsg {
    pub fn format(self) -> StdResult<Binary> {
        match self {
            // If both set, receiving contract must handle it using the ModuleInitMsg
            ModuleInitMsg {
                fixed_init: Some(_),
                root_init: Some(_),
            } => to_binary(&self),
            // If not, we can simplify by only sending the custom or fixed message.
            ModuleInitMsg {
                fixed_init: None,
                root_init: Some(r),
            } => Ok(r),
            ModuleInitMsg {
                fixed_init: Some(f),
                root_init: None,
            } => Ok(f),
            ModuleInitMsg {
                fixed_init: None,
                root_init: None,
            } => Err(StdError::generic_err("No init msg set for this module")),
        }
    }
}
