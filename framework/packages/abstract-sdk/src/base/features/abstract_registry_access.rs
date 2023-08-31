use crate::{feature_objects::VersionControlContract, AbstractSdkResult};
use cosmwasm_std::Deps;

/// Trait that enables access to a registry, like a version control contract.
pub trait AbstractRegistryAccess: Sized {
    /// Get the address of the registry.
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<VersionControlContract>;
}
