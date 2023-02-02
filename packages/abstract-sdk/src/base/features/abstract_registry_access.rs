use cosmwasm_std::{Addr, Deps, StdResult};

/// Trait that enables access to a registry, like a version control contract.
pub trait AbstractRegistryAccess: Sized {
    /// Get the address of the registry.
    fn abstract_registry(&self, deps: Deps) -> StdResult<Addr>;
}
