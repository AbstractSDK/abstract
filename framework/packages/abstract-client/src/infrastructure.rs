//! # Represents chain infrastructure
//!
//! [`Environment`] allows you to get the execution environment of the object.
//!
//! You might want to do this to get the cw-orchestrator type of the infrastructure which enables you to
//! call some environment-specific methods or do low-level operations.
//!
//! You also sometimes need to provide the environment as a parameter to some methods, e.g. when you want to deploy a contract.

use abstract_interface::{Abstract, AbstractInterfaceError};
use cw_orch::prelude::*;

use crate::{account::Account, AbstractClient};

use cw_orch::environment::Environment as _;

/// Trait for retrieving the CosmWasm environment that is being used.
pub trait Environment<Chain: CwEnv> {
    /// Get the execution environment
    fn environment(&self) -> Chain;
}

pub(crate) trait Infrastructure<Chain: CwEnv>: Environment<Chain> {
    /// Get the infrastructure on the execution environment
    fn infrastructure(&self) -> Result<Abstract<Chain>, AbstractInterfaceError> {
        let chain = self.environment();
        Abstract::load_from(chain)
    }
}

impl<Chain: CwEnv, T> Infrastructure<Chain> for T where T: Environment<Chain> {}

impl<Chain: CwEnv> Environment<Chain> for Account<Chain> {
    fn environment(&self) -> Chain {
        self.abstr_account.environment().clone()
    }
}

impl<Chain: CwEnv> Environment<Chain> for AbstractClient<Chain> {
    fn environment(&self) -> Chain {
        self.abstr.registry.environment().clone()
    }
}
