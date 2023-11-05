use std::ops::Deref;

use abstract_core::{
    module_factory::ModuleInstallConfig,
    objects::module::{ModuleInfo, ModuleVersion},
    objects::namespace::Namespace,
    AbstractResult,
};
use abstract_interface::{Abstract, AbstractAccount, Manager, ManagerExecFns};
use cosmwasm_std::to_json_binary;
use cw_orch::prelude::*;
use cw_orch::{contract::Contract, prelude::*};
use serde::Serialize;

use crate::{application::Application, infrastructure::Infrastructure};

pub struct AccountBuilder {}

pub struct Account<T: CwEnv> {
    pub(crate) account: AbstractAccount<T>,
}

impl<T: CwEnv> Account<T> {
    // Install an application on the account
    // creates a new sub-account and installs the application on it.
    // TODO: For abstract we know that the contract's name in cw-orch = the module's name in abstract.
    // So we should be able to create the module (M) from only the type and the chain (T).
    pub fn install_app<M: ContractInstance<T>, C: Serialize>(
        &self,
        id: &str,
        configuration: &C,
        funds: Vec<Coin>,
    ) -> AbstractResult<Application<T, M>> {
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
                    Some(to_json_binary(&configuration)?),
                )],
                &funds,
            )
            .unwrap();
        // Construct module from type and chain
        let contract = Contract::new(id.to_string(), self.environment());
        // TODO: convert contract to M here
        Ok(Application::new(self.account, contract))
    }
}

pub struct InterchainAccount {}
