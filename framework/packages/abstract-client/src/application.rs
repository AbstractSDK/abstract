use std::ops::Deref;

use abstract_interface::AbstractAccount;
use cw_orch::prelude::*;
use serde::Serialize;

// An application represents a module installed on a (sub)-account.
pub struct Application<T: CwEnv, M> {
    account: AbstractAccount<T>,
    module: M,
}

// Allows to access the module's methods directly from the application struct
impl<T: CwEnv, M> Deref for Application<T, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<T: CwEnv, M> Application<T, M> {
    pub fn new(account: AbstractAccount<T>, module: M) -> Self {
        Self { account, module }
    }
}

// pub trait Installable<T: CwEnv> {
//     fn install<C: Serialize >(&self, account: AbstractAccount<T>, configuration: C) -> Abstract<T> {

//     };
// }
