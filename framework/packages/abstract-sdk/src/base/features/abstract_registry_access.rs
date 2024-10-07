use abstract_std::objects::registry::RegistryContract;
use cosmwasm_std::{Deps, Env};

use crate::AbstractSdkResult;

/// Trait that enables access to a registry, like a registry contract.
pub trait AbstractRegistryAccess: Sized {
    /// Get the address of the registry.
    fn abstract_registry(&self, deps: Deps, env: &Env) -> AbstractSdkResult<RegistryContract>;
}
