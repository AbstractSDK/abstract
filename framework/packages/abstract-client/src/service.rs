//! # Represents Abstract Service
//!
//! [`Service`] represents a module registered in version control

use std::marker::PhantomData;

use abstract_interface::{RegisteredModule, VersionControl};
use abstract_std::objects::{module::ModuleInfo, module_reference::ModuleReference};
use cw_orch::{contract::Contract, prelude::*};

use crate::client::AbstractClientResult;

/// An service represents a module registered in version control.
///
/// It implements cw-orch traits of the module itself, so you can call its methods directly from the service struct.
#[derive(Clone)]
pub struct Service<T: CwEnv, M> {
    module: M,
    chain: PhantomData<T>,
}

/// Allows to access the module's methods directly from the service struct
impl<Chain: CwEnv, M: InstantiableContract + ContractInstance<Chain>> InstantiableContract
    for Service<Chain, M>
{
    type InstantiateMsg = M::InstantiateMsg;
}

impl<Chain: CwEnv, M: QueryableContract + ContractInstance<Chain>> QueryableContract
    for Service<Chain, M>
{
    type QueryMsg = M::QueryMsg;
}

impl<Chain: CwEnv, M: ExecutableContract + ContractInstance<Chain>> ExecutableContract
    for Service<Chain, M>
{
    type ExecuteMsg = M::ExecuteMsg;
}

impl<Chain: CwEnv, M: MigratableContract + ContractInstance<Chain>> MigratableContract
    for Service<Chain, M>
{
    type MigrateMsg = M::MigrateMsg;
}

impl<Chain: CwEnv, M: ContractInstance<Chain>> ContractInstance<Chain> for Service<Chain, M> {
    fn as_instance(&self) -> &Contract<Chain> {
        self.module.as_instance()
    }

    fn as_instance_mut(&mut self) -> &mut Contract<Chain> {
        self.module.as_instance_mut()
    }
}

impl<Chain: CwEnv, M: RegisteredModule + From<Contract<Chain>>> Service<Chain, M> {
    /// Get module interface installed from version control
    pub(crate) fn new(version_control: &VersionControl<Chain>) -> AbstractClientResult<Self> {
        // The module must be in version control and service
        let module_reference: ModuleReference = version_control
            .module(ModuleInfo::from_id(
                M::module_id(),
                abstract_std::objects::module::ModuleVersion::Version(
                    M::module_version().to_owned(),
                ),
            )?)?
            .reference;
        let ModuleReference::Service(service_addr) = module_reference else {
            return Err(crate::AbstractClientError::ExpectedService {});
        };

        // Ensure using correct address
        let contract = Contract::new(M::module_id(), version_control.environment().clone());
        contract.set_address(&service_addr);

        Ok(Self {
            module: contract.into(),
            chain: PhantomData {},
        })
    }
}
