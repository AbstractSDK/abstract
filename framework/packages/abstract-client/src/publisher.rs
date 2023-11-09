use abstract_core::objects::AssetEntry;
use abstract_interface::{AppDeployer, DeployStrategy, ModuleId};
use cosmwasm_std::{Addr, Coin};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv, InstantiableContract},
};
use semver::Version;
use serde::Serialize;

use crate::{
    account::{Account, AccountBuilder},
    application::Application,
    client::AbstractClientResult,
    infrastructure::Infrastructure,
};

pub struct PublisherBuilder<'a, Chain: CwEnv> {
    account_builder: AccountBuilder<'a, Chain>,
}

impl<'a, Chain: CwEnv> PublisherBuilder<'a, Chain> {
    pub(crate) fn new(account_builder: AccountBuilder<'a, Chain>) -> Self {
        Self { account_builder }
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        Self {
            account_builder: self.account_builder.name(name),
        }
    }

    pub fn description(self, description: impl Into<String>) -> Self {
        Self {
            account_builder: self.account_builder.description(description),
        }
    }

    pub fn link(self, link: impl Into<String>) -> Self {
        Self {
            account_builder: self.account_builder.link(link),
        }
    }

    pub fn namespace(self, namespace: impl Into<String>) -> Self {
        Self {
            account_builder: self.account_builder.namespace(namespace),
        }
    }

    pub fn base_asset(self, base_asset: AssetEntry) -> Self {
        Self {
            account_builder: self.account_builder.base_asset(base_asset),
        }
    }

    pub fn build(self) -> AbstractClientResult<Publisher<Chain>> {
        let account = self.account_builder.build()?;
        Ok(Publisher { account })
    }
}

/// A publisher represents an account that owns a namespace with the goal of publishing software to the module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    pub(crate) fn new(account: Account<Chain>) -> Self {
        Self { account }
    }

    pub fn install_app<
        M: ContractInstance<Chain> + ModuleId + InstantiableContract + From<Contract<Chain>> + Clone,
        C: Serialize,
    >(
        &self,
        configuration: &C,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        self.account.install_app(configuration, funds)
    }

    pub fn deploy_module<
        M: ContractInstance<Chain>
            + ModuleId
            + InstantiableContract
            + From<Contract<Chain>>
            + AppDeployer<Chain>,
    >(
        &self,
        version: Version,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id(), self.account.environment());
        let app: M = contract.into();
        app.deploy(version, DeployStrategy::Try).map_err(Into::into)
    }

    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    // TODO: handle error
    pub fn admin(&self) -> AbstractClientResult<Addr> {
        self.account
            .abstr_account
            .manager
            .address()
            .map_err(Into::into)
    }
}
