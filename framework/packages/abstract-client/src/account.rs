use abstract_core::{
    module_factory::ModuleInstallConfig,
    objects::module::{ModuleInfo, ModuleVersion},
    objects::namespace::Namespace,
    AbstractResult,
};
use abstract_interface::{AbstractAccount, ManagerExecFns};
use cosmwasm_std::to_binary;
use cw_orch::prelude::*;
use serde::Serialize;

use crate::{application::Application, infrastructure::Infrastructure};

pub struct AccountBuilder {}

pub struct Account<T: CwEnv> {
    pub(crate) account: AbstractAccount<T>,
}

pub struct Contract<T: CwEnv> {
    pub(crate) contract: cw_orch::contract::Contract<T>,
}

impl<T: CwEnv> Contract<T> {
    fn new(id: String, environment: T) -> Self {
        Self {
            contract: cw_orch::contract::Contract::new(id, environment),
        }
    }
}

impl<T: CwEnv> ContractInstance<T> for Contract<T> {
    fn as_instance(&self) -> &cw_orch::contract::Contract<T> {
        &self.contract
    }

    fn as_instance_mut(&mut self) -> &mut cw_orch::contract::Contract<T> {
        &mut self.contract
    }
}

impl<T: CwEnv + 'static> Account<T> {
    // Install an application on the account
    // creates a new sub-account and installs the application on it.
    // TODO: For abstract we know that the contract's name in cw-orch = the module's name in abstract.
    // So we should be able to create the module (M) from only the type and the chain (T).
    pub fn install_app<C: Serialize>(
        &self,
        id: &str,
        configuration: &C,
        funds: Vec<Coin>,
    ) -> AbstractResult<Application<T>> {
        self.account
            .manager
            .install_modules(
                vec![ModuleInstallConfig::new(
                    ModuleInfo {
                        // TODO: Set name and namespace properly.
                        name: "name".to_string(),
                        namespace: Namespace::new("namespace")?,
                        version: ModuleVersion::Latest,
                    },
                    Some(to_binary(&configuration)?),
                )],
                &funds,
            )
            .unwrap();
        // Construct module from type and chain
        let contract: Box<dyn ContractInstance<T>> =
            Box::new(Contract::new(id.to_string(), self.environment()));
        Ok(Application::new(&self.account, contract))
    }
}

pub struct InterchainAccount {}
