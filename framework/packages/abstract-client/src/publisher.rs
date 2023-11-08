use abstract_core::{
    objects::namespace::Namespace, version_control::NamespaceResponse, AbstractResult,
};
use abstract_interface::{
    Abstract, AbstractAccount, AppDeployer, DeployStrategy, ModuleId, VCQueryFns,
};
use cosmwasm_std::{Addr, Coin};
use cw_orch::{
    contract::Contract,
    prelude::{ContractInstance, CwEnv, InstantiableContract},
};
use semver::Version;
use serde::Serialize;

use crate::{account::Account, application::Application, infrastructure::Infrastructure};

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
