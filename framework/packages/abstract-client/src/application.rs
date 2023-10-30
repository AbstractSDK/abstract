use std::ops::Deref;

use abstract_interface::AbstractAccount;
use cw_orch::prelude::*;

type Module<T> = Box<dyn ContractInstance<T>>;

// An application represents a module installed on a (sub)-account.
pub struct Application<'a, T: CwEnv + 'a> {
    account: &'a AbstractAccount<T>,
    module: Module<T>,
}

// Allows to access the module's methods directly from the application struct
impl<'a, T: CwEnv> Deref for Application<'a, T> {
    type Target = Module<T>;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<'a, T: CwEnv> Application<'a, T> {
    pub(crate) fn new(
        account: &'a AbstractAccount<T>,
        module: Box<dyn ContractInstance<T>>,
    ) -> Self {
        Self { account, module }
    }
}

// pub trait Installable<T: CwEnv> {
//     fn install<C: Serialize >(&self, account: AbstractAccount<T>, configuration: C) -> Abstract<T> {

//     };
// }
