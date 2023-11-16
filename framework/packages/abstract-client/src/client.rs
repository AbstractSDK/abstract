use abstract_interface::{Abstract, AnsHost};
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

    pub fn name_service(&self) -> &AnsHost<Chain> {
        &self.abstr.ans_host
    }

    pub fn get_publisher_from_namespace(
        &self,
        namespace: String,
    ) -> AbstractClientResult<Publisher<Chain>> {
        Ok(Publisher::new(self.get_account_from_namespace(namespace)?))
    }

    pub fn publisher_builder(&self) -> PublisherBuilder<Chain> {
        PublisherBuilder::new(AccountBuilder::new(&self.abstr))
    }

    pub fn account_builder(&self) -> AccountBuilder<Chain> {
        AccountBuilder::new(&self.abstr)
    }

    pub fn get_account_from_namespace(
        &self,
        namespace: String,
    ) -> AbstractClientResult<Account<Chain>> {
        Account::from_namespace(&self.abstr, namespace)
    }
}
