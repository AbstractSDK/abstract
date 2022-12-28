use abstract_os::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
    },
    version_control::{state::MODULE_LIBRARY, ModuleResponse, QueryMsg},
};
use cosmwasm_std::{Deps, StdError};

use crate::helpers::cosmwasm_std::wasm_smart_query;
use cosmwasm_std::StdResult;

use super::RegisterAccess;

/// Access the Abstract Version Register to query module information.
pub trait VersionRegisterInterface: RegisterAccess {
    fn version_register<'a>(&'a self, deps: Deps<'a>) -> VersionRegister<Self> {
        VersionRegister { base: self, deps }
    }
}

impl<T> VersionRegisterInterface for T where T: RegisterAccess {}

#[derive(Clone)]
pub struct VersionRegister<'a, T: VersionRegisterInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: VersionRegisterInterface> VersionRegister<'a, T> {
    pub fn query_module_reference_raw(
        &self,
        module_info: ModuleInfo,
    ) -> StdResult<ModuleReference> {
        let registry_addr = self.base.registry(self.deps)?;
        MODULE_LIBRARY
            .query(
                &self.deps.querier,
                registry_addr.clone(),
                module_info.clone(),
            )?
            .ok_or_else(|| {
                StdError::generic_err(format!(
                    "Module {} can not be found in registry {}.",
                    module_info, registry_addr
                ))
            })
    }

    /// Smart query for a module
    pub fn query_module(&self, module_info: ModuleInfo) -> StdResult<Module> {
        let registry_addr = self.base.registry(self.deps)?;
        let ModuleResponse { module } = self.deps.querier.query(&wasm_smart_query(
            registry_addr.into_string(),
            &QueryMsg::Module {
                module: module_info,
            },
        )?)?;
        Ok(module)
    }
}
