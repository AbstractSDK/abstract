use std::ops::Deref;
use std::ops::DerefMut;

use abstract_core::manager::ManagerModuleInfo;
use abstract_interface::RegisteredModule;
use cw_orch::contract::Contract;
use cw_orch::prelude::*;

use crate::account::Account;
use crate::client::AbstractClientResult;
use crate::error::AbstractClientError;
use crate::infrastructure::Infrastructure;

/// An application represents a module installed on a (sub)-account.
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

impl<Chain: CwEnv, M> Application<Chain, M> {
    pub fn new(account: Account<Chain>, module: M) -> Self {
        Self { account, module }
    }

    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    /// Attempts to get a module on the application. This would typically be a dependency of the
    /// module of type `M`.
    pub fn module<T: RegisteredModule + From<Contract<Chain>>>(&self) -> AbstractClientResult<T> {
        let module_id = T::module_id();
        let module_info: Option<ManagerModuleInfo> =
            self.account.abstr_account.manager.module_info(module_id)?;
        if let Some(module_info) = module_info {
            let contract = Contract::new(module_id.to_owned(), self.account.environment())
                .with_address(Some(&module_info.address));

            let module: T = contract.into();
            Ok(module)
        } else {
            Err(AbstractClientError::ModuleNotInstalled {})
        }
    }
}
