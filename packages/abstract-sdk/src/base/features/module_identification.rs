pub trait ModuleIdentification: Sized {
    fn module_id(&self) -> &'static str;
}
