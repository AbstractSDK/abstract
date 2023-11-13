use abstract_core::{
    manager::{state::AccountInfo, InfoResponse},
    objects::{gov_type::GovernanceDetails, namespace::Namespace, AssetEntry},
    version_control::NamespaceResponse,
};
use abstract_interface::{
    Abstract, AbstractAccount, AccountDetails, ManagerQueryFns, RegisteredModule, VCQueryFns,
};
use cw_orch::contract::Contract;
use cw_orch::prelude::*;
use serde::Serialize;

use crate::{
    application::Application, client::AbstractClientResult, infrastructure::Infrastructure,
};

pub struct AccountBuilder<'a, Chain: CwEnv> {
    pub(crate) abstr: &'a Abstract<Chain>,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    base_asset: Option<AssetEntry>,
    // TODO: Decide if we want to abstract this as well.
    governance_details: Option<GovernanceDetails<String>>,
    // TODO: How to handle install_modules?
}

impl<'a, Chain: CwEnv> AccountBuilder<'a, Chain> {
    pub(crate) fn new(abstr: &'a Abstract<Chain>) -> Self {
        Self {
            abstr,
            name: None,
            description: None,
            link: None,
            namespace: None,
            base_asset: None,
            governance_details: None,
        }
    }

    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = Some(name.into());
        self
    }

    pub fn description(&mut self, description: impl Into<String>) -> &mut Self {
        self.description = Some(description.into());
        self
    }

    pub fn link(&mut self, link: impl Into<String>) -> &mut Self {
        self.link = Some(link.into());
        self
    }

    pub fn namespace(&mut self, namespace: impl Into<String>) -> &mut Self {
        self.namespace = Some(namespace.into());
        self
    }

    pub fn base_asset(&mut self, base_asset: AssetEntry) -> &mut Self {
        self.base_asset = Some(base_asset);
        self
    }

    pub fn governance_details(
        &mut self,
        governance_details: GovernanceDetails<String>,
    ) -> &mut Self {
        self.governance_details = Some(governance_details);
        self
    }

    pub fn build(&self) -> AbstractClientResult<Account<Chain>> {
        let sender = self.environment().sender().to_string();
        let name = self
            .name
            .clone()
            .unwrap_or_else(|| String::from("Default Abstract Account"));
        let governance_details = self
            .governance_details
            .clone()
            .unwrap_or(GovernanceDetails::Monarchy { monarch: sender });
        let abstract_account = self.abstr.account_factory.create_new_account(
            AccountDetails {
                name,
                description: self.description.clone(),
                link: self.link.clone(),
                namespace: self.namespace.clone(),
                base_asset: self.base_asset.clone(),
                install_modules: vec![],
            },
            governance_details,
            Some(&[]),
        )?;
        Ok(Account::new(abstract_account))
    }
}

pub struct Account<Chain: CwEnv> {
    pub(crate) abstr_account: AbstractAccount<Chain>,
}

impl<Chain: CwEnv> Account<Chain> {
    pub(crate) fn new(abstract_account: AbstractAccount<Chain>) -> Self {
        Self {
            abstr_account: abstract_account,
        }
    }

    pub(crate) fn from_namespace(
        abstr: &Abstract<Chain>,
        namespace: String,
    ) -> AbstractClientResult<Self> {
        let namespace_response: NamespaceResponse = abstr
            .version_control
            .namespace(Namespace::new(&namespace)?)?;

        let abstract_account: AbstractAccount<Chain> =
            AbstractAccount::new(abstr, Some(namespace_response.account_id));

        Ok(Self::new(abstract_account))
    }

    pub fn get_account_info(&self) -> AbstractClientResult<AccountInfo<Addr>> {
        let info_response: InfoResponse = self.abstr_account.manager.info()?;
        Ok(info_response.info)
    }

    // Install an application on the account
    // creates a new sub-account and installs the application on it.
    pub fn install_app<
        M: ContractInstance<Chain>
            + RegisteredModule
            + InstantiableContract
            + From<Contract<Chain>>
            + Clone,
        C: Serialize,
    >(
        &self,
        configuration: &C,
        funds: &[Coin],
    ) -> AbstractClientResult<Application<Chain, M>> {
        let contract = Contract::new(M::module_id().to_owned(), self.environment());

        let app: M = contract.into();

        self.abstr_account
            .install_app(app.clone(), configuration, Some(funds))?;
        Ok(Application::new(self.abstr_account.clone(), app))
    }
}

pub struct InterchainAccount {}
