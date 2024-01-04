//! # Represents chain infrastructure
//!
//! [`Environment`] allows you to get chain environment of the object

use abstract_interface::{Abstract, AbstractInterfaceError};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use crate::{account::Account, client::AbstractClient};

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
        self.abstr_account.proxy.get_chain().clone()
    }
}

impl<Chain: CwEnv> Environment<Chain> for AbstractClient<Chain> {
    fn environment(&self) -> Chain {
        self.abstr.version_control.get_chain().clone()
    }
}
