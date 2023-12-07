use crate::core::objects::module::ModuleId;

use crate::base::Handler;

/// Return the identifier for this module.
pub trait ModuleIdentification: Sized {
    /// Get the module identifier.
    fn module_id(&self) -> ModuleId;
}

impl<T: Handler> ModuleIdentification for T {
    fn module_id(&self) -> ModuleId {
        self.contract().info().0
    }
}
