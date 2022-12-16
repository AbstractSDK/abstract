//! # Application
//! The Application interface provides helper functions to execute functions on other applications installed on the OS.

use abstract_os::{
    api::{BaseExecuteMsg, ExecuteMsg},
    manager::state::{ModuleId, OS_MODULES},
};
use cosmwasm_std::{
    wasm_execute, Addr, CosmosMsg, Deps, Empty, QueryRequest, StdError, StdResult, WasmQuery,
};
use cw2::{ContractVersion, CONTRACT};
use serde::Serialize;

use super::{Dependencies, Identification};

/// Interact with other applications on the OS.
pub trait ApplicationInterface: Identification + Dependencies {
    fn applications<'a>(&'a self, deps: Deps<'a>) -> Applications<Self> {
        Applications { base: self, deps }
    }
}

impl<T> ApplicationInterface for T where T: Identification + Dependencies {}

#[derive(Clone)]
pub struct Applications<'a, T: ApplicationInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ApplicationInterface> Applications<'a, T> {
    /// Retrieve the address of an application in this OS.
    /// This should **not** be used to execute messages on an `Api`.
    /// Use `Applications::api_request(..)` instead.
    pub fn app_address(&self, module_id: ModuleId) -> StdResult<Addr> {
        let manager_addr = self.base.manager_address(self.deps)?;
        let maybe_module_addr = OS_MODULES.query(&self.deps.querier, manager_addr, module_id)?;
        let Some(module_addr) = maybe_module_addr else {
            return Err(StdError::generic_err(format!("Module {} not enabled on OS.",module_id)));
        };
        Ok(module_addr)
    }

    /// RawQuery the version of an enabled module
    pub fn app_version(&self, app_id: ModuleId) -> StdResult<ContractVersion> {
        let app_address = self.app_address(app_id)?;
        let req = QueryRequest::Wasm(WasmQuery::Raw {
            contract_addr: app_address.into(),
            key: CONTRACT.as_slice().into(),
        });
        self.deps.querier.query::<ContractVersion>(&req)
    }

    /// Construct an api request message.
    pub fn api_request<M: Serialize>(
        &self,
        api_id: ModuleId,
        message: impl Into<ExecuteMsg<M, Empty>>,
    ) -> StdResult<CosmosMsg> {
        self.assert_app_is_dependency(api_id)?;
        let api_msg: ExecuteMsg<M, Empty> = message.into();
        let api_address = self.app_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    /// Construct an API configure message
    pub fn configure_api(&self, api_id: ModuleId, message: BaseExecuteMsg) -> StdResult<CosmosMsg> {
        let api_msg: ExecuteMsg<Empty, Empty> = message.into();
        let api_address = self.app_address(api_id)?;
        Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
    }

    fn assert_app_is_dependency(&self, app: ModuleId) -> StdResult<()> {
        let is_app_dependencies = Dependencies::dependencies(self.base)
            .iter()
            .map(|d| d.id)
            .any(|x| x == app);
        if !is_app_dependencies {
            return Err(StdError::generic_err(format!(
                "Module {} not defined as dependency on this module.",
                app
            )));
        }
        Ok(())
    }
}
