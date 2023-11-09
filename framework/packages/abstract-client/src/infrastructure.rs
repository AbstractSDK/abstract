use abstract_interface::{Abstract, AbstractInterfaceError};
use cw_orch::deploy::Deploy;
use cw_orch::prelude::*;

use crate::account::{Account, AccountBuilder};

pub(crate) trait Infrastructure<T: CwEnv> {
    // Get the execution environment
    fn environment(&self) -> T;

    // Get the infrastructure on the execution environment
    fn infrastructure(&self) -> Result<Abstract<T>, AbstractInterfaceError> {
        let chain = self.environment();
        Abstract::load_from(chain)
    }
}

impl<Chain: CwEnv> Infrastructure<Chain> for Account<Chain> {
    fn environment(&self) -> Chain {
        self.abstr_account.proxy.get_chain().clone()
    }
}

impl<'a, Chain: CwEnv> Infrastructure<Chain> for AccountBuilder<'a, Chain> {
    fn environment(&self) -> Chain {
        self.abstr.account.proxy.get_chain().clone()
    }
}
