use crate::base::features::ModuleIdentification;
use crate::{base::Handler, AbstractSdkError};
use abstract_std::ibc::IbcResponseMsg;
use abstract_std::IBC_CLIENT;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response, StdError};

/// Trait for a contract's IBC callback ExecuteMsg variant.
pub trait IbcCallbackEndpoint: Handler {
    /// Queries the IBC Client address.
    fn ibc_client_addr(&self, deps: Deps) -> Result<Addr, Self::Error>;

    /// Handler for the `ExecuteMsg::IbcCallback()` variant.
    fn ibc_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::Error> {
        // Make sure module have ibc_client as dependency
        if !self
            .dependencies()
            .iter()
            .any(|static_dep| static_dep.id == IBC_CLIENT)
        {
            return Err(AbstractSdkError::Std(StdError::generic_err(format!(
                "Ibc Client is not dependency of {}",
                self.module_id()
            )))
            .into());
        }

        let ibc_client = self.ibc_client_addr(deps.as_ref())?;

        if info.sender.ne(&ibc_client) {
            return Err(AbstractSdkError::CallbackNotCalledByIbcClient {
                caller: info.sender,
                client_addr: ibc_client,
                module: self.info().0.to_string(),
            }
            .into());
        };
        let ibc_callback_handler =
            self.maybe_ibc_callback_handler()
                .ok_or(AbstractSdkError::NoModuleIbcHandler(
                    self.module_id().to_string(),
                ))?;

        ibc_callback_handler(deps, env, info, self, msg.callback, msg.result)
    }
}
