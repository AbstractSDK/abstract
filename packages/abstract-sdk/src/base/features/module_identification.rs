/// Return the identifier for this module.
pub trait ModuleIdentification: Sized {
    /// Get the module identifier.
    fn module_id(&self) -> &'static str;
}
