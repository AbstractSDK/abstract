//! # Module
//! The Module interface provides helper functions to execute functions on other modules installed on the OS.

use abstract_os::{
    api, app,
    manager::state::{ModuleId, OS_MODULES},
};
use cosmwasm_std::{
    wasm_execute, Addr, CosmosMsg, Deps, Empty, QueryRequest, StdError, StdResult, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use serde::Serialize;

use super::{Dependencies, Identification};

/// Interact with other modules on the OS.
pub trait ModuleInterface: Identification + Dependencies {
    fn modules<'a>(&'a self, deps: Deps<'a>) -> Modules<Self> {
        Modules { base: self, deps }
    }
}

impl<T> ModuleInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Modules<'a, T: ModuleInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleInterface> Modules<'a, T> {
    /// Retrieve the address of an application in this OS.
    /// This should **not** be used to execute messages on an `Api`.
    /// Use `Modules::api_request(..)` instead.
    pub fn module_address(&self, module_id: ModuleId) -> StdResult<Addr> {
        let manager_addr = self.base.manager_address(self.deps)?;
        let maybe_module_addr = OS_MODULES.query(&self.deps.querier, manager_addr, module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(StdError::generic_err(format!("Module {} not enabled on OS.", module_id)));
        };
        Ok(module_addr)
    }

    /// RawQuery the version of an enabled module
    pub fn module_version(&self, module_id: ModuleId) -> StdResult<ContractVersion> {
        let module_address = self.module_address(module_id)?;
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: module_address.into(),
            key: CONTRACT.as_slice().into(),
        });
        self.deps.querier.query::<ContractVersion>(&req)
    }

    /// Construct an api request message.
    pub fn api_request<M: Serialize>(
        &self,
        api_id: ModuleId,
        message: impl Into<api::ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        self.assert_module_dependency(api_id)?;
        let api_msg: api::ExecuteMsg<M, Empty> = message.into();
        let api_address = self.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Construct an API configure message
    pub fn configure_api(
        &self,
        api_id: ModuleId,
        message: api::BaseExecuteMsg,
    ) -> StdResult<CosmosMsg> {
        let api_msg: api::ExecuteMsg<Empty, Empty> = message.into();
        let api_address = self.module_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Construct an api request message.
    pub fn app_request<M: Serialize>(
        &self,
        app_id: ModuleId,
        message: impl Into<app::ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        self.assert_module_dependency(app_id)?;
        let app_msg: app::ExecuteMsg<M, Empty> = message.into();
        let app_address = self.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    /// Construct an API configure message
    pub fn configure_app(
        &self,
        app_id: ModuleId,
        message: app::BaseExecuteMsg,
    ) -> StdResult<CosmosMsg> {
        let app_msg: app::ExecuteMsg<Empty, Empty> = message.into();
        let app_address = self.module_address(app_id)?;
        Ok(wasm_execute(app_address, &app_msg, vec![])?.into())
    }

    fn assert_module_dependency(&self, module_id: ModuleId) -> StdResult<()> {
        let is_dependency = Dependencies::dependencies(self.base)
            .iter()
            .map(|d| d.id)
            .any(|x| x == module_id);

        match is_dependency {
            true => Ok(()),
            false => Err(StdError::generic_err(format!(
                "Module {} is not a dependency of this contract.",
                module_id
            ))),
        }
    }
}
