use abstract_core::objects::dependency::{Dependency};
use abstract_sdk::AbstractSdkError;


/// Retrieve the dependencies of a module.
pub trait Dependencies: Sized {
    /// Get the dependencies of the module.
    fn dependencies(&self) -> Result<Vec<Dependency>, AbstractSdkError>;
}