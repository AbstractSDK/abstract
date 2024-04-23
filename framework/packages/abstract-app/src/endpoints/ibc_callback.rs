use abstract_sdk::ModuleInterface;
use abstract_std::IBC_CLIENT;
use cosmwasm_std::Addr;

use crate::{state::ContractError, AppContract, IbcCallbackEndpoint};

impl<
        Error: ContractError,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > IbcCallbackEndpoint
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
    fn ibc_client(&self, deps: cosmwasm_std::Deps) -> Result<Addr, Self::Error> {
        let ibc_client = self.modules(deps).module_address(IBC_CLIENT)?;
        Ok(ibc_client)
    }
}
