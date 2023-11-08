use abstract_core::{
    objects::{gov_type::GovernanceDetails, namespace::Namespace, AssetEntry},
    version_control::NamespaceResponse,
    AbstractResult,
};
use abstract_interface::{
    Abstract, AbstractAccount, AccountDetails, AppDeployer, DeployStrategy, ModuleId, VCQueryFns,
};
use cosmwasm_std::{Addr, Coin};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv, InstantiableContract},
};
use semver::Version;
use serde::Serialize;

use crate::{account::Account, application::Application, infrastructure::Infrastructure};

pub struct PublisherBuilder<'a, Chain: CwEnv> {
    abstr: &'a Abstract<Chain>,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    base_asset: Option<AssetEntry>,
    governance_details: GovernanceDetails<String>,
}

impl<'a, Chain: CwEnv> PublisherBuilder<'a, Chain> {
    pub(crate) fn new(
        abstr: &'a Abstract<Chain>,
        governance_details: GovernanceDetails<String>,
    ) -> Self {
        Self {
            abstr,
            name: None,
            description: None,
            link: None,
            namespace: None,
            base_asset: None,
            governance_details,
        }
    }

    pub fn name(self, name: impl Into<String>) -> Self {
        Self {
            name: Some(name.into()),
            ..self
        }
    }

    pub fn description(self, description: impl Into<String>) -> Self {
        Self {
            description: Some(description.into()),
            ..self
        }
    }

    pub fn link(self, link: impl Into<String>) -> Self {
        Self {
            link: Some(link.into()),
            ..self
        }
    }

    pub fn namespace(self, namespace: impl Into<String>) -> Self {
        Self {
            namespace: Some(namespace.into()),
            ..self
        }
    }

    pub fn base_asset(self, base_asset: AssetEntry) -> Self {
        Self {
            base_asset: Some(base_asset),
            ..self
        }
    }

    pub fn build(self) -> Publisher<Chain> {
        let abstract_account: AbstractAccount<Chain> = if let Some(name) = self.name {
            self.abstr
                .account_factory
                .create_new_account(
                    AccountDetails {
                        name,
                        description: self.description,
                        link: self.link,
                        namespace: self.namespace,
                        base_asset: self.base_asset,
                        install_modules: vec![],
                    },
                    self.governance_details,
                    &[],
                )
                .unwrap()
        } else {
            self.abstr
                .account_factory
                .create_default_account(self.governance_details)
                .unwrap()
        };
        Publisher::new(abstract_account)
    }
}

// A provider represents an account that owns a namespace with the goal of publishing software to the module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    pub(crate) fn new_existing_publisher(abstr: &Abstract<Chain>, namespace: String) -> Self {
        let namespace_response: Result<NamespaceResponse, cw_orch::prelude::CwOrchError> = abstr
            .version_control
            .namespace(Namespace::new(&namespace).unwrap());

        let abstract_account: AbstractAccount<Chain> =
            AbstractAccount::new(abstr, Some(namespace_response.unwrap().account_id));

        Self::new(abstract_account)
    }

    fn new(abstr_account: AbstractAccount<Chain>) -> Self {
        Self {
            account: Account::new(abstr_account),
        }
    }

    pub fn install_app<
        M: ContractInstance<Chain> + ModuleId + InstantiableContract + From<Contract<Chain>> + Clone,
        C: Serialize,
    >(
        &self,
        configuration: &C,
        funds: &[Coin],
    ) -> AbstractResult<Application<Chain, M>> {
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
    ) {
        let contract = Contract::new(M::module_id(), self.account.environment());
        let app: M = contract.into();
        app.deploy(version, DeployStrategy::Try).unwrap();
    }

    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    // TODO: handle error
    pub fn admin(&self) -> Addr {
        self.account.abstr_account.manager.address().unwrap()
    }
}
