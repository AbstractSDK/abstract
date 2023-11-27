use cosmwasm_std::{Addr, StdError};

/// Store the Version Control contract.
/// Implements [`AbstractRegistryAccess`]
#[cosmwasm_schema::cw_serde]
pub struct VersionControlContract {
    /// Address of the version control contract
    pub address: Addr,
}

impl VersionControlContract {
    /// Construct a new version control feature object.
    pub fn new(address: Addr) -> Self {
        Self { address }
    }

    /// Raw query for a module reference
    pub fn query_module_reference_raw(
        &self,
        module_info: &ModuleInfo,
        deps: Querier,
    ) -> StdResult<ModuleReference> {
        REGISTERED_MODULES
            .query(&self.deps.querier, self.address.clone(), module_info)?
            .ok_or_else(|| StdError::ModuleNotFound {
                module: module_info.to_string(),
                registry_addr,
            })
    }
}
