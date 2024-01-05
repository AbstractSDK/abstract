//! # Represents Abstract Application
//!
//! [`Application`] represents a module installed on a (sub-)account

use std::ops::Deref;
use std::ops::DerefMut;

use abstract_interface::ManagerQueryFns;
use abstract_interface::RegisteredModule;
use cw_orch::contract::Contract;
use cw_orch::prelude::*;

use crate::account::Account;
use crate::client::AbstractClientResult;
use crate::error::AbstractClientError;
use crate::infrastructure::Environment;

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
    pub fn new(account: Account<Chain>, module: M) -> AbstractClientResult<Self> {
        // Sanity check: the module must be installed on the account
        account.module_addresses(vec![M::module_id().to_string()])?;
        Ok(Self { account, module })
    }

    /// Sub-account on which application is installed
    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }

    /// Attempts to get a module on the application. This would typically be a dependency of the
    /// module of type `M`.
    pub fn module<T: RegisteredModule + From<Contract<Chain>>>(&self) -> AbstractClientResult<T> {
        let module_id = T::module_id();
        let maybe_module_addr = self
            .account
            .abstr_account
            .manager
            .module_addresses(vec![module_id.to_string()])?
            .modules;
        if !maybe_module_addr.is_empty() {
            let contract = Contract::new(module_id.to_owned(), self.account.environment())
                .with_address(Some(&maybe_module_addr[0].1));
            let module: T = contract.into();
            Ok(module)
        } else {
            Err(AbstractClientError::ModuleNotInstalled {})
        }
    }
}
