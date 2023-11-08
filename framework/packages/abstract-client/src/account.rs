use abstract_core::{
    objects::{gov_type::GovernanceDetails, AssetEntry},
    AbstractResult,
};
use abstract_interface::{Abstract, AbstractAccount, AccountDetails, ModuleId};
use cw_orch::contract::Contract;
use cw_orch::prelude::*;
use serde::Serialize;

use crate::{application::Application, infrastructure::Infrastructure};

pub struct AccountBuilder<'a, Chain: CwEnv> {
    abstr: &'a Abstract<Chain>,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    base_asset: Option<AssetEntry>,
    governance_details: GovernanceDetails<String>,
    // TODO: How to handle install_modules?
}

impl<'a, Chain: CwEnv> AccountBuilder<'a, Chain> {
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

    pub fn build(self) -> Account<Chain> {
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
        Account::new(abstract_account)
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
}

impl<Chain: CwEnv> Account<Chain> {
    // Install an application on the account
    // creates a new sub-account and installs the application on it.
    // TODO: For abstract we know that the contract's name in cw-orch = the module's name in abstract.
    // So we should be able to create the module (M) from only the type and the chain (T).
    pub fn install_app<
        // Not sure about this From<Contract<Chain>>
        M: ContractInstance<Chain> + ModuleId + InstantiableContract + From<Contract<Chain>> + Clone,
        C: Serialize,
    >(
        &self,
        configuration: &C,
        funds: &[Coin],
    ) -> AbstractResult<Application<Chain, M>> {
        let contract = Contract::new(M::module_id(), self.environment());

        let app: M = contract.into();

        self.abstr_account
            .install_app(app.clone(), configuration, funds)
            .unwrap();
        Ok(Application::new(self.abstr_account.clone(), app))
    }
}

pub struct InterchainAccount {}
