use abstract_core::ibc::ModuleIbcMsg;
use cosmwasm_std::{Addr, Deps, DepsMut, Env, MessageInfo, Response};

use crate::{base::Handler, AbstractSdkError};

/// Trait for a contract to call itself on an IBC counterpart.
pub trait ModuleIbcEndpoint: Handler {
    /// Get the address of the ibc host associated with this module
    fn ibc_host(&self, deps: Deps) -> Result<Addr, Self::Error>;

    /// Handler for the `ExecuteMsg::IbcCallback()` variant.
    fn module_ibc(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ModuleIbcMsg,
    ) -> Result<Response, Self::Error> {
        // Only an IBC host can call this endpoint
        let ibc_host = self.ibc_host(deps.as_ref())?;
        if info.sender.ne(&ibc_host) {
            return Err(AbstractSdkError::ModuleIbcNotCalledByHost {
                caller: info.sender,
                host_addr: ibc_host,
                module: self.info().0.to_string(),
            }
            .into());
        };

        // Only a whitelisted chain can call this endpoint
        /// TODO
        ///
        ///
        let maybe_handler = self.maybe_module_ibc_handler();
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |handler| handler(deps, env, self, msg),
        )
    }
}
