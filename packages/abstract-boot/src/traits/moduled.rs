// use abstract_core::objects::module::{ModuleInfo, ModuleVersion};
// use abstract_core::objects::module_reference::ModuleReference;
// use boot_core::interface::{ContractInstance, CwInterface};
// use boot_core::{BootEnvironment, Contract};

pub trait IntoModule<Chain> {
    // fn latest<RefFn>(&self, ref_fn: RefFn) -> (ModuleInfo, ModuleReference)
    // where
    //     RefFn: Fn(&&Contract<Chain>) -> ModuleReference,
    // {
    //     (
    //         ModuleInfo::from_id(&self.id, ModuleVersion::Latest)?,
    //         ref_fn(self),
    //     )
    // }
}

// impl<T, Chain: BootEnvironment> IntoModule<Chain> for T
// where
//     T: ContractInstance<Chain> + CwInterface,
//     Chain: BootEnvironment,
// {
// }
