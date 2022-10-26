use abstract_os::{
    abstract_ica::{IbcResponseMsg, StdAck},
    objects::UncheckedContractEntry,
    IBC_CLIENT,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};

use crate::MemoryOperation;

pub type IbcCallbackHandlerFn<Module, Error> =
    fn(DepsMut, Env, MessageInfo, Module, String, StdAck) -> Result<Response, Error>;

pub trait IbcCallbackEndpoint: Sized + MemoryOperation {
    type ContractError: From<cosmwasm_std::StdError>;
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    fn callback_handler(&self, id: &str)
        -> Option<IbcCallbackHandlerFn<Self, Self::ContractError>>;
    fn handle_ibc_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::ContractError> {
        // Todo: Change to use version control instead?
        let ibc_client = self.resolve(
            deps.as_ref(),
            &UncheckedContractEntry::try_from(IBC_CLIENT.to_string())?.check(),
        )?;
        if info.sender.ne(&ibc_client) {
            return Err(StdError::GenericErr {
                msg: format! {"ibc callback can only be called by local ibc client {}",ibc_client },
            }
            .into());
        }
        let IbcResponseMsg { id, msg: ack } = msg;
        let maybe_handler = self.callback_handler(&id);
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |f| f(deps, env, info, self, id, ack),
        )
    }
}
