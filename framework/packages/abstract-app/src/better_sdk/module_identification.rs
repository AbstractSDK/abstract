use abstract_core::objects::module::ModuleId;
use abstract_sdk::AbstractSdkError;

/// Return the identifier for this module.
pub trait ModuleIdentification: Sized {
    /// Get the module identifier.
    fn module_id(&self) -> Result<String, AbstractSdkError>;
}
