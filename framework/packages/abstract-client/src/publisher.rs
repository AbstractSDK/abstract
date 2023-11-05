use abstract_interface::AbstractAccount;
use cosmwasm_std::{Addr, Coin};
use cw_orch::prelude::{ContractInstance, CwEnv};

use crate::account::Account;

pub struct PublisherBuilder {}

// A provider represents an account that owns a namespace with the goal of publishing software to the module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    /*pub(crate) fn new(namespace: String) -> Self {
        //let account = ...?
        Self { namespace }
    }*/

    fn publish() {}

    fn account(&self) -> &Account<Chain> {
        &self.account
    }

    // TODO: handle error
    fn admin(&self) -> Addr {
        self.account.account.manager.address().unwrap()
    }
}
