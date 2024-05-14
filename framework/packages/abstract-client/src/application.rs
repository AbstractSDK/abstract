//! # Represents Abstract Application
//!
//! [`Application`] represents a module installed on a (sub-)account

use std::ops::{Deref, DerefMut};

use abstract_interface::RegisteredModule;
use cw_orch::{contract::Contract, prelude::*};

use crate::{account::Account, client::AbstractClientResult, infrastructure::Infrastructure};

/// An application represents a module installed on a (sub)-[`Account`].
///
/// It derefs to the module itself, so you can call its methods directly from the application struct.
pub struct Application<T: CwEnv, M> {
    account: Account<T>,
    module: M,
}

/// Allows to access the module's methods directly from the application struct
impl<Chain: CwEnv, M> Deref for Application<Chain, M> {
    type Target = M;

    fn deref(&self) -> &Self::Target {
        &self.module
    }
}

impl<Chain: CwEnv, M> DerefMut for Application<Chain, M> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.module
    }
}

impl<Chain: CwEnv, M: RegisteredModule> Application<Chain, M> {
    /// Get module interface installed on provided account
    pub(crate) fn new(account: Account<Chain>, module: M) -> AbstractClientResult<Self> {
        // Sanity check: the module must be installed on the account
        account.module_addresses(vec![M::module_id().to_string()])?;
        // figure out if contract is adapter or app
        let module = account
            .infrastructure()?
            .version_control
            .module(M::module_info()?)?;
        let execute_through_manager = match module {
            Module::Adapter(_) => true,
            Module::App(_) => false,
        };
        Ok(Self { account, module })
    }

    /// Sub-account on which application is installed
    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    /// Attempts to get a module on the application. This would typically be a dependency of the
    /// module of type `M`.
    pub fn module<T: RegisteredModule + From<Contract<Chain>>>(&self) -> AbstractClientResult<T> {
        self.account.module()
    }
}

impl<Chain: CwEnv, M: ContractInstance<Chain>> Application<Chain, M> {
    /// Authorize this application on installed adapters. Accepts Module Id's of adapters
    pub fn authorize_on_adapters(&self, adapter_ids: &[&str]) -> AbstractClientResult<()> {
        for module_id in adapter_ids {
            self.account
                .abstr_account
                .manager
                .update_adapter_authorized_addresses(module_id, vec![self.addr_str()?], vec![])?;
        }
        Ok(())
    }
}
