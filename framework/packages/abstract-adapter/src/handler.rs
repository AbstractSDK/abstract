use crate::{state::ContractError, AdapterContract};
use abstract_sdk::base::{AbstractContract, Handler};
use cosmwasm_std::Empty;

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, ReceiveMsg, SudoMsg> Handler
    for AdapterContract<Error, InitMsg, ExecMsg, QueryMsg, ReceiveMsg, SudoMsg>
{
    type Error = Error;
    type CustomInitMsg = InitMsg;
    type CustomExecMsg = ExecMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = Empty;
    type ReceiveMsg = ReceiveMsg;
    type SudoMsg = SudoMsg;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
