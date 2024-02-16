use abstract_core::IBC_HOST;
use abstract_sdk::{base::ModuleIbcEndpoint, ModuleInterface};

use crate::{state::ContractError, AppContract};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > ModuleIbcEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn ibc_host(&self, deps: cosmwasm_std::Deps) -> Result<cw_orch::prelude::Addr, Self::Error> {
        let ibc_client = self.modules(deps).module_address(IBC_HOST)?;
        Ok(ibc_client)
    }
}
