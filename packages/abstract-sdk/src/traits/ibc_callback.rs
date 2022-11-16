use abstract_os::{abstract_ica::IbcResponseMsg, objects::UncheckedContractEntry, IBC_CLIENT};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response, StdError};

use crate::{AnsHostOperation, Handler};

pub trait IbcCallbackEndpoint: Handler + AnsHostOperation {
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    fn handle_ibc_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::Error> {
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
        let maybe_handler = self.maybe_ibc_callback_handler(&id);
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |f| f(deps, env, info, self, id, ack),
        )
    }
}
