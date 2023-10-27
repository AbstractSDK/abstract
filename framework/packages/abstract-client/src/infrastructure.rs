use abstract_interface::{Abstract, AbstractAccount, AbstractInterfaceError};
use cw_orch::prelude::*;

use crate::account::Account;

pub(crate) trait Infrastructure<T: CwEnv> {
    // Get the execution environment
    fn environment(&self) -> T;

    // Get the infrastructure on the execution environment
    fn infrastructure(&self) -> Result<Abstract<T>, AbstractInterfaceError> {
        let chain = self.environment();
        Abstract::load_from(chain)
    }
}

impl<M: CwEnv> Infrastructure<M> for Account<M> {
    fn environment(&self) -> M {
        self.account.proxy.get_chain().clone()
    }
}
