//! # Represents Abstract Publisher
//!
//! [`Publisher`] is an Account with helpers for publishing and maintaining Abstract Applications and Adapters

use abstract_interface::{
    AdapterDeployer, AppDeployer, DeployStrategy, RegisteredModule, RegistryQueryFns,
    ServiceDeployer, StandaloneDeployer,
};
use cw_orch::{contract::Contract, prelude::*};
use serde::Serialize;

use crate::{
    account::Account, client::AbstractClientResult, infrastructure::Infrastructure,
    AbstractClientError, Environment,
};

/// A Publisher represents an account that owns a namespace with the goal of publishing modules to the on-chain module-store.
pub struct Publisher<Chain: CwEnv> {
    account: Account<Chain>,
}

impl<Chain: CwEnv> Publisher<Chain> {
    /// New publisher from account. We check that the account has an associated namespace.
    /// If you create a publisher from an account without a namespace, use [`Self::new_with_namespace`] to claim it
    pub fn new(account: &Account<Chain>) -> AbstractClientResult<Self> {
        let namespace = account
            .infrastructure()?
            .registry
            .namespaces(vec![account.id()?])?;

        if namespace.namespaces.is_empty() {
            return Err(AbstractClientError::NoNamespace {
                account: account.id()?,
            });
        }

        Ok(Self {
            account: account.clone(),
        })
    }

    /// New publisher from account and namespace
    /// Claim the namespace from the account
    pub fn new_with_namespace(
        account: Account<Chain>,
        namespace: &str,
    ) -> AbstractClientResult<Self> {
        account.claim_namespace(namespace)?;
        Ok(Self { account })
    }

    /// Publish an Abstract App
    pub fn publish_app<
        M: ContractInstance<Chain> + RegisteredModule + From<Contract<Chain>> + AppDeployer<Chain>,
    >(
        &self,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let app: M = contract.into();
        app.deploy(M::module_version().parse()?, DeployStrategy::Try)
            .map_err(Into::into)
    }

    /// Publish an Abstract Standalone
    pub fn publish_standalone<
        M: ContractInstance<Chain>
            + RegisteredModule
            + From<Contract<Chain>>
            + StandaloneDeployer<Chain>,
    >(
        &self,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let standalone: M = contract.into();
        standalone
            .deploy(M::module_version().parse()?, DeployStrategy::Try)
            .map_err(Into::into)
    }

    /// Publish an Abstract Adapter
    pub fn publish_adapter<
        CustomInitMsg: Serialize,
        M: ContractInstance<Chain>
            + RegisteredModule
            + From<Contract<Chain>>
            + AdapterDeployer<Chain, CustomInitMsg>,
    >(
        &self,
        init_msg: CustomInitMsg,
    ) -> AbstractClientResult<M> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let adapter: M = contract.into();
        adapter.deploy(M::module_version().parse()?, init_msg, DeployStrategy::Try)?;
        Ok(adapter)
    }

    /// Publish an Abstract Service
    pub fn publish_service<
        M: ContractInstance<Chain> + RegisteredModule + From<Contract<Chain>> + ServiceDeployer<Chain>,
    >(
        &self,
        init_msg: &<M as InstantiableContract>::InstantiateMsg,
    ) -> AbstractClientResult<()> {
        let contract = Contract::new(M::module_id().to_owned(), self.account.environment());
        let service: M = contract.into();
        service
            .deploy(M::module_version().parse()?, init_msg, DeployStrategy::Try)
            .map_err(Into::into)
    }

    /// Abstract Account of the publisher
    pub fn account(&self) -> &Account<Chain> {
        &self.account
    }
}
