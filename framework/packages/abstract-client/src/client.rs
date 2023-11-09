use abstract_interface::Abstract;
use cw_orch::{deploy::Deploy, prelude::CwEnv};

use crate::{
    account::{Account, AccountBuilder},
    error::AbstractClientError,
    publisher::{Publisher, PublisherBuilder},
};

pub struct AbstractClient<Chain: CwEnv> {
    abstr: Abstract<Chain>,
}

pub type AbstractClientResult<T> = Result<T, AbstractClientError>;

impl<Chain: CwEnv> AbstractClient<Chain> {
    pub fn new(chain: Chain) -> AbstractClientResult<Self> {
        let abstr = Abstract::load_from(chain)?;
        Ok(Self { abstr })
    }

    pub fn existing_publisher(&self, namespace: String) -> AbstractClientResult<Publisher<Chain>> {
        Ok(Publisher::new(self.new_existing_account(namespace)?))
    }

    pub fn new_publisher(&self) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr))
    }

    pub fn new_account(&self) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr)
    }

    pub fn new_existing_account(&self, namespace: String) -> AbstractClientResult<Account<Chain>> {
        Account::new_existing_account(&self.abstr, namespace)
    }
}
