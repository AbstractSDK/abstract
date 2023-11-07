use abstract_core::{objects::namespace::Namespace, version_control::NamespaceResponse};
use abstract_interface::{Abstract, AbstractAccount, VCQueryFns};
use cosmwasm_std::{Addr, Coin};
use cw_orch::prelude::{ContractInstance, CwEnv};

use crate::account::Account;

pub struct PublisherBuilder {}

// A provider represents an account that owns a namespace with the goal of publishing software to the module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    pub(crate) fn new(abstr: &Abstract<Chain>, namespace: String) -> Self {
        let namespace_response: Result<NamespaceResponse, cw_orch::prelude::CwOrchError> = abstr
            .version_control
            .namespace(Namespace::new(&namespace).unwrap());

        let abstract_account: AbstractAccount<Chain> =
            AbstractAccount::new(abstr, Some(namespace_response.unwrap().account_id));

        // TODO: add logic for when namespace does not exist.
        Self {
            account: Account::new(abstract_account),
        }
    }

    pub fn publish() {}

    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    // TODO: handle error
    pub fn admin(&self) -> Addr {
        self.account.account.manager.address().unwrap()
    }
}
