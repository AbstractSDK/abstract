use std::ops::Deref;

use abstract_interface::{AbstractAccount, Manager, Abstract, ManagerExecFns};
use cw_orch::{prelude::*, contract::Contract};
use serde::Serialize;

use crate::{application::Application, infrastructure::Infrastructure};

pub struct AccountBuilder {
}

pub struct Account<T: CwEnv> {
    pub(crate) account: AbstractAccount<T>,
}

impl<T: CwEnv> Account<T> {
    // Install an application on the account
    // creates a new sub-account and installs the application on it.
    // TODO: For abstract we know that the contract's name in cw-orch = the module's name in abstract.
    // So we should be able to create the module (M) from only the type and the chain (T).
    pub fn install_app<M: ContractInstance<T>, C: Serialize>(&self,id: &str ,configuration: &C, funds: Vec<Coin>) -> Application<T, M> {
        
        self.account.manager.install_modules(vec![ModuleInstallConfig {
            id: id.to_string(),
            configuration: Some(configuration),
        }], &funds).unwrap();
        // Construct module from type and chain
        let contract = Contract::new(id, self.environment());
        // TODO: convert contract to M here
        Application::new(account, contract)
    }

    
}

pub struct InterchainAccount {}
