use abstract_core::{
    module_factory::ModuleInstallConfig,
    objects::module::{ModuleInfo, ModuleVersion},
    objects::namespace::Namespace,
    AbstractResult,
};
use abstract_interface::{AbstractAccount, ManagerExecFns};
use cosmwasm_std::to_json_binary;
use cw_orch::contract::Contract;
use cw_orch::prelude::*;
use serde::Serialize;

use crate::{application::Application, infrastructure::Infrastructure};

pub struct AccountBuilder {}

pub struct Account<Chain: CwEnv> {
    pub(crate) account: AbstractAccount<Chain>,
}

pub trait ModuleId {
    fn module_id() -> String;
}

impl<Chain: CwEnv> Account<Chain> {
    // Install an application on the account
    // creates a new sub-account and installs the application on it.
    // TODO: For abstract we know that the contract's name in cw-orch = the module's name in abstract.
    // So we should be able to create the module (M) from only the type and the chain (T).
    pub fn install_app<
        // Not sure about this From<Contract<Chain>>
        M: ContractInstance<Chain> + ModuleId + InstantiableContract + From<Contract<Chain>>,
        C: Serialize,
    >(
        &self,
        configuration: &M::InstantiateMsg,
        funds: Vec<Coin>,
    ) -> AbstractResult<Application<Chain, M>> {
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
        let contract = Contract::new(M::module_id(), self.environment());

        let app: M = contract.into();

        // TODO: convert contract to M here
        Ok(Application::new(self.account.clone(), app))
    }
}

pub struct InterchainAccount {}
