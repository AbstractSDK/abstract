use crate::base::Handler;
use os::objects::dependency::StaticDependency;

/// Retrieve the dependencies of a module.
pub trait Dependencies: Sized {
    fn dependencies(&self) -> &[StaticDependency];
}

impl<T: Handler> Dependencies for T {
    fn dependencies(&self) -> &[StaticDependency] {
        Handler::dependencies(self)
    }
}
