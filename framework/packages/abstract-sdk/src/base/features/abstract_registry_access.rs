use crate::AbstractSdkResult;
use abstract_core::objects::version_control::VersionControlContract;

/// Trait that enables access to a registry, like a version control contract.
pub trait AbstractRegistryAccess: Sized {
    /// Get the address of the registry.
    fn abstract_registry(&self) -> AbstractSdkResult<VersionControlContract>;
}
