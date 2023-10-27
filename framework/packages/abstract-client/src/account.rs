use std::ops::Deref;

use abstract_interface::{AbstractAccount, Manager};
use cw_orch::prelude::*;

pub struct AccountBuilder {}

pub struct Account<T: CwEnv> {
    account: AbstractAccount<T>,
}

impl<T: CwEnv> Account<T> {}

pub struct InterchainAccount {}