use std::ops::Deref;

use abstract_interface::AbstractAccount;
use cw_orch::prelude::*;

// An application represents a module installed on a (sub)-account.
pub struct Application<T: CwEnv, M> {
    _account: AbstractAccount<T>,
    module: M,
}

// Allows to access the module's methods directly from the application struct
impl<Chain: CwEnv, M> Deref for Application<Chain, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<T: CwEnv, M> Application<T, M> {
    pub fn new(account: AbstractAccount<T>, module: M) -> Self {
        Self {
            _account: account,
            module,
        }
    }
}

// pub trait Installable<T: CwEnv> {
//     fn install<C: Serialize >(&self, account: AbstractAccount<T>, configuration: C) -> Abstract<T> {

//     };
// }
