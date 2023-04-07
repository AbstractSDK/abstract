use crate::{state::ContractError, ApiContract};
use abstract_sdk::base::{AbstractContract, Handler};
use cosmwasm_std::Empty;

impl<Error: ContractError, InitMsg, ExecMsg, QueryMsg, SudoMsg, Receive> Handler
    for ApiContract<Error, InitMsg, ExecMsg, QueryMsg, SudoMsg, Receive>
{
    type Error = Error;
    type CustomExecMsg = ExecMsg;
    type CustomInitMsg = InitMsg;
    type CustomQueryMsg = QueryMsg;
    type CustomMigrateMsg = Empty;
    type SudoMsg = SudoMsg;
    type ReceiveMsg = Receive;

    fn contract(&self) -> &AbstractContract<Self, Error> {
        &self.contract
    }
}
