use std::ops::Deref;

use cw_orch::prelude::*;

use crate::account::Account;

// An application represents a module installed on a (sub)-account.
pub struct Application<T: CwEnv, M> {
    account: Account<T>,
    module: M,
}

// Allows to access the module's methods directly from the application struct
impl<Chain: CwEnv, M> Deref for Application<Chain, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<Chain: CwEnv, M> Application<Chain, M> {
    pub fn new(account: Account<Chain>, module: M) -> Self {
        Self { account, module }
    }

    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }
}

// pub trait Installable<T: CwEnv> {
//     fn install<C: Serialize >(&self, account: AbstractAccount<T>, configuration: C) -> Abstract<T> {

//     };
// }
