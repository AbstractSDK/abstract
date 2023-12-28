use abstract_core::{
    manager::{ModuleAddressesResponse, ModuleInfosResponse},
    objects::{gov_type::GovernanceDetails, AssetEntry},
};
use abstract_interface::{
    AdapterDeployer, AppDeployer, DependencyCreation, DeployStrategy, InstallConfig,
    RegisteredModule,
};
use cosmwasm_std::{Addr, Coin};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv},
};
use serde::Serialize;

use crate::{
    account::{Account, AccountBuilder},
    application::Application,
    client::AbstractClientResult,
    infrastructure::Environment,
};

pub struct PublisherBuilder<'a, Chain: CwEnv> {
    account_builder: AccountBuilder<'a, Chain>,
}

impl<'a, Chain: CwEnv> PublisherBuilder<'a, Chain> {
    pub(crate) fn new(account_builder: AccountBuilder<'a, Chain>) -> Self {
        Self { account_builder }
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.account_builder.name(name);
        self
    }

    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.account_builder.description(description);
        self
    }

    pub fn link(&mut self, link: impl Into<String>) -> &mut Self {
        self.account_builder.link(link);
        self
    }

    pub fn namespace(&mut self, namespace: impl Into<String>) -> &mut Self {
        self.account_builder.namespace(namespace);
        self
    }

    pub fn base_asset(&mut self, base_asset: AssetEntry) -> &mut Self {
        self.account_builder.base_asset(base_asset);
        self
    }

    pub fn governance_details(
        &mut self,
        governance_details: GovernanceDetails<String>,
    ) -> &mut Self {
        self.account_builder.governance_details(governance_details);
        self
    }

    pub fn build(&self) -> AbstractClientResult<Publisher<Chain>> {
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
        M: ContractInstance<Chain> + InstallConfig + From<Contract<Chain>> + Clone,
    >(
        &self,
        configuration: &M::InitMsg,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        self.account.install_app(configuration, funds)
    }

    pub fn install_app_with_dependencies<
        M: ContractInstance<Chain>
            + DependencyCreation
            + InstallConfig
            + From<Contract<Chain>>
            + Clone,
    >(
        &self,
        module_configuration: &M::InitMsg,
        dependencies_config: M::DependenciesConfig,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        self.account
            .install_app_with_dependencies(module_configuration, dependencies_config, funds)
    }

    pub fn publish_app<
        M: ContractInstance<Chain> + RegisteredModule + From<Contract<Chain>> + AppDeployer<Chain>,
    >(
        &self,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let app: M = contract.into();
        app.deploy(M::module_version().parse()?, DeployStrategy::Try)
            .map_err(Into::into)
    }

    pub fn deploy_adapter<
        CustomInitMsg: Serialize,
        M: ContractInstance<Chain>
            + RegisteredModule
            + From<Contract<Chain>>
            + AdapterDeployer<Chain, CustomInitMsg>,
    >(
        &self,
        init_msg: CustomInitMsg,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let app: M = contract.into();
        app.deploy(M::module_version().parse()?, init_msg, DeployStrategy::Try)
            .map_err(Into::into)
    }

    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }
    // TODO: add `account_admin` fn to get the (Sub-)Account's admin.
    pub fn admin(&self) -> AbstractClientResult<Addr> {
        self.account.manager()
    }

    pub fn proxy(&self) -> AbstractClientResult<Addr> {
        self.account.proxy()
    }

    pub fn module_infos(&self) -> AbstractClientResult<ModuleInfosResponse> {
        self.account.module_infos()
    }

    pub fn module_addresses(
        &self,
        ids: Vec<String>,
    ) -> AbstractClientResult<ModuleAddressesResponse> {
        self.account.module_addresses(ids)
    }
}
