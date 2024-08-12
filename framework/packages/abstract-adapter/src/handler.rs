use abstract_sdk::base::{AbstractContract, Handler};
use cosmwasm_std::Empty;

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, SudoMsg> Handler
    for AdapterContract<Error, InitMsg, ExecMsg, QueryMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = Empty;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
