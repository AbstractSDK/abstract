use crate::base::Handler;
use os::objects::dependency::StaticDependency;

pub trait Dependencies: Sized {
    fn dependencies(&self) -> &[StaticDependency];
}

impl<T: Handler> Dependencies for T {
    fn dependencies(&self) -> &[StaticDependency] {
        Handler::dependencies(self)
    }
}
