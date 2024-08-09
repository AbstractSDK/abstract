use abstract_sdk::base::{AbstractContract, Handler};
use cosmwasm_std::Empty;

use crate::{state::ContractError, AdapterContract};

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, UntaggedMsg, SudoMsg> Handler
    for AdapterContract<Error, InitMsg, ExecMsg, QueryMsg, UntaggedMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = Empty;
    type UntaggedMsg = UntaggedMsg;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
