use super::AbstractRegistryAccess;
use crate::{helpers::cosmwasm_std::wasm_smart_query, AbstractSdkError, AbstractSdkResult};
use abstract_os::{
    objects::{
        module::{Module, ModuleInfo},
        module_reference::ModuleReference,
    },
    version_control::{state::MODULE_LIBRARY, ModulesResponse, QueryMsg},
};
use cosmwasm_std::Deps;

/// Access the Abstract Version Control and access the modules.
pub trait ModuleRegistryInterface: AbstractRegistryAccess {
    fn module_registry<'a>(&'a self, deps: Deps<'a>) -> ModuleRegistry<Self> {
        ModuleRegistry { base: self, deps }
    }
}

impl<T> ModuleRegistryInterface for T where T: AbstractRegistryAccess {}

#[derive(Clone)]
pub struct ModuleRegistry<'a, T: ModuleRegistryInterface> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: ModuleRegistryInterface> ModuleRegistry<'a, T> {
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
    ) -> AbstractSdkResult<ModuleReference> {
        let registry_addr = self.base.abstract_registry(self.deps)?;
        MODULE_LIBRARY
            .query(&self.deps.querier, registry_addr.clone(), module_info)?
            .ok_or_else(|| AbstractSdkError::ModuleNotFound {
                module: module_info.to_string(),
                registry_addr,
            })
    }

    /// Smart query for a module
    pub fn query_module(&self, module_info: ModuleInfo) -> AbstractSdkResult<Module> {
        let registry_addr = self.base.abstract_registry(self.deps)?;
        let ModulesResponse { mut modules } = self.deps.querier.query(&wasm_smart_query(
            registry_addr.into_string(),
            &QueryMsg::Modules {
                infos: vec![module_info],
            },
        )?)?;
        Ok(modules.swap_remove(0))
    }
}
