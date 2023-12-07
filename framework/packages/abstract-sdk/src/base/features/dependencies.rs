use crate::base::Handler;
use crate::core::objects::dependency::Dependency;

/// Retrieve the dependencies of a module.
pub trait Dependencies: Sized {
    /// Get the dependencies of the module.
    fn dependencies(&self) -> Vec<Dependency>;
}

impl<T: Handler> Dependencies for T {
    fn dependencies(&self) -> Vec<Dependency> {
        Handler::dependencies(self)
    }
}
