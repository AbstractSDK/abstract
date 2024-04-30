use crate::{base::Handler, std::objects::dependency::StaticDependency};

/// Retrieve the dependencies of a module.
pub trait Dependencies: Sized {
    /// Get the dependencies of the module.
    fn dependencies(&self) -> &[StaticDependency];
}

impl<T: Handler> Dependencies for T {
    fn dependencies(&self) -> &[StaticDependency] {
        Handler::dependencies(self)
    }
}
