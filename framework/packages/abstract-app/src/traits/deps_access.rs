use abstract_sdk::features::DepsAccess;
use cosmwasm_std::{Env, MessageInfo};

use crate::{state::ContractError, AppContract};

/// The state variables for our AppContract.
impl<
        'app,
        T: DepsAccess,
        Error: ContractError,
        CustomInitMsg: 'static,
        CustomExecMsg: 'static,
        CustomQueryMsg: 'static,
        CustomMigrateMsg: 'static,
        ReceiveMsg: 'static,
        SudoMsg: 'static,
    > DepsAccess
    for AppContract<
        'app,
        T,
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn deps_mut<'a: 'b, 'b>(&'a mut self) -> cosmwasm_std::DepsMut<'b> {
        self.deps.deps_mut()
    }

    fn deps<'a: 'b, 'b>(&'a self) -> cosmwasm_std::Deps<'b> {
        self.deps.deps()
    }

    fn env(&self) -> Env {
        self.deps.env()
    }

    fn message_info(&self) -> MessageInfo {
        self.deps.message_info()
    }
}
