use abstract_std::objects::version_control::VersionControlContract;
use cosmwasm_std::Deps;

use crate::AbstractSdkResult;

/// Trait that enables access to a registry, like a version control contract.
pub trait AbstractRegistryAccess: Sized {
    /// Get the address of the registry.
    fn abstract_registry(&self, deps: Deps) -> AbstractSdkResult<VersionControlContract>;
}
