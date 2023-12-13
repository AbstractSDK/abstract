use crate::base::Handler;

/// Return the identifier for this module.
pub trait ModuleIdentification: Sized {
    /// Get the module identifier.
    fn module_id(&self) -> &str;
}

impl<T: Handler> ModuleIdentification for T {
    fn module_id(&self) -> &str {
        self.contract().info().0
    }
}
