//! # Represents Abstract Application
//!
//! [`Application`] represents a module installed on a (sub-)account

use abstract_interface::RegisteredModule;
use cw_orch::{contract::Contract, prelude::*};

use crate::{account::Account, client::AbstractClientResult};

/// An application represents a module installed on a (sub)-[`Account`].
///
/// It implements cw-orch traits of the module itself, so you can call its methods directly from the application struct.
#[derive(Clone)]
pub struct Application<T: CwEnv, M> {
    account: Account<T>,
    module: M,
}

/// Allows to access the module's methods directly from the application struct
impl<Chain: CwEnv, M: InstantiableContract + ContractInstance<Chain>> InstantiableContract
    for Application<Chain, M>
{
    type InstantiateMsg = M::InstantiateMsg;
}

impl<Chain: CwEnv, M: QueryableContract + ContractInstance<Chain>> QueryableContract
    for Application<Chain, M>
{
    type QueryMsg = M::QueryMsg;
}

impl<Chain: CwEnv, M: ExecutableContract + ContractInstance<Chain>> ExecutableContract
    for Application<Chain, M>
{
    type ExecuteMsg = M::ExecuteMsg;
}

impl<Chain: CwEnv, M: MigratableContract + ContractInstance<Chain>> MigratableContract
    for Application<Chain, M>
{
    type MigrateMsg = M::MigrateMsg;
}

impl<Chain: CwEnv, M: ContractInstance<Chain>> ContractInstance<Chain> for Application<Chain, M> {
    fn as_instance(&self) -> &Contract<Chain> {
        self.module.as_instance()
    }

    fn as_instance_mut(&mut self) -> &mut Contract<Chain> {
        self.module.as_instance_mut()
    }
}

impl<Chain: CwEnv, M: RegisteredModule> Application<Chain, M> {
    /// Get module interface installed on provided account
    pub(crate) fn new(account: Account<Chain>, module: M) -> AbstractClientResult<Self> {
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
