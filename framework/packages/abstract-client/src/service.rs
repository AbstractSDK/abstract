//! # Represents Abstract Service
//!
//! [`Service`] represents a module registered in registry

use std::marker::PhantomData;

use abstract_interface::{RegisteredModule, Registry};
use abstract_std::objects::{module::ModuleInfo, module_reference::ModuleReference};
use cw_orch::{contract::Contract, prelude::*};

use crate::{client::AbstractClientResult, Application};

/// A `Service` represents a contract registered in registry.
///
/// `Service`s should be created from [`Application`]s using the `into_service` method.
/// They can then be registered using the `service.deploy()` method.
//
// It implements cw-orch traits of the module itself, so you can call its methods directly from the service struct.
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

impl<Chain: CwEnv, M: ContractInstance<Chain>> ContractInstance<Chain> for Service<Chain, M> {
    fn as_instance(&self) -> &Contract<Chain> {
        self.module.as_instance()
    }

    fn as_instance_mut(&mut self) -> &mut Contract<Chain> {
        self.module.as_instance_mut()
    }
}

impl<Chain: CwEnv, M: RegisteredModule + From<Contract<Chain>>> Service<Chain, M> {
    /// Get module interface installed from registry
    pub(crate) fn new(registry: &Registry<Chain>) -> AbstractClientResult<Self> {
        // The module must be in registry and service
        let module_reference: ModuleReference = registry
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
        let contract = Contract::new(M::module_id(), registry.environment().clone());
        contract.set_address(&service_addr);

        Ok(Self {
            module: contract.into(),
            chain: PhantomData {},
        })
    }
}

impl<T: CwEnv, M> From<Application<T, M>> for Service<T, M> {
    fn from(value: Application<T, M>) -> Self {
        Self {
            module: value.module,
            chain: PhantomData::<T> {},
        }
    }
}
