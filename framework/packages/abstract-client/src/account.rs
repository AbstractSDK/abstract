use abstract_core::AbstractResult;
use abstract_interface::{AbstractAccount, ModuleId};
use cw_orch::contract::Contract;
use cw_orch::prelude::*;
use serde::Serialize;

use crate::{application::Application, infrastructure::Infrastructure};

pub struct AccountBuilder {}

pub struct Account<Chain: CwEnv> {
    pub(crate) account: AbstractAccount<Chain>,
}

impl<Chain: CwEnv> Account<Chain> {
    pub(crate) fn new(abstract_account: AbstractAccount<Chain>) -> Self {
        Self {
            account: abstract_account,
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

        self.account
            .install_app(app.clone(), configuration, funds)
            .unwrap();

        Ok(Application::new(self.account.clone(), app))
    }
}

pub struct InterchainAccount {}
