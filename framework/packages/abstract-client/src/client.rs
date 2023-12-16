use abstract_interface::{Abstract, AnsHost, VersionControl};
use cosmwasm_std::{Addr, BlockInfo, Coin, Uint128};
use cw_orch::{deploy::Deploy, environment::MutCwEnv, prelude::CwEnv};

use crate::{
    account::{Account, AccountBuilder},
    error::AbstractClientError,
    infrastructure::{Environment, Infrastructure},
    publisher::{Publisher, PublisherBuilder},
};

pub struct AbstractClient<Chain: CwEnv> {
    pub(crate) abstr: Abstract<Chain>,
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

    pub fn version_control(&self) -> &VersionControl<Chain> {
        &self.abstr.version_control
    }

    pub fn block_info(&self) -> AbstractClientResult<BlockInfo> {
        self.environment()
            .block_info()
            .map_err(Into::<cw_orch::prelude::CwOrchError>::into)
            .map_err(Into::<AbstractClientError>::into)
    }

    pub fn get_publisher_from_namespace(
        &self,
        namespace: &str,
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
        namespace: &str,
    ) -> AbstractClientResult<Account<Chain>> {
        Account::from_namespace(&self.abstr, namespace)
    }

    pub fn sender(&self) -> Addr {
        self.environment().sender()
    }

    pub fn query_balance(
        &self,
        address: &Addr,
        denom: impl Into<String>,
    ) -> AbstractClientResult<Uint128> {
        let coins = self.balance(address, Some(denom.into()))?;
        // There will always be a single element in this case.
        Ok(coins[0].amount)
    }

    pub fn query_balances(&self, address: &Addr) -> AbstractClientResult<Vec<Coin>> {
        self.balance(address, None)
    }

    pub fn wait_blocks(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_blocks(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn wait_seconds(&self, amount: u64) -> AbstractClientResult<()> {
        self.environment()
            .wait_seconds(amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn next_block(&self) -> AbstractClientResult<()> {
        self.environment()
            .next_block()
            .map_err(Into::into)
            .map_err(Into::into)
    }
}

impl<Chain: MutCwEnv> AbstractClient<Chain> {
    pub fn set_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .set_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }

    pub fn add_balance(&self, address: &Addr, amount: Vec<Coin>) -> AbstractClientResult<()> {
        self.environment()
            .add_balance(address, amount)
            .map_err(Into::into)
            .map_err(Into::into)
    }
}
