use crate::core::AbstractError;
use crate::{base::Handler, features::AbstractRegistryAccess, AbstractSdkError, ModuleInterface};
use abstract_core::objects::module_reference::ModuleReference;
use abstract_core::{ibc::IbcResponseMsg, objects::module::ModuleInfo, IBC_CLIENT};
use cosmwasm_std::{DepsMut, Env, MessageInfo, Response};

/// Trait for a contract's IBC callback ExecuteMsg variant.
pub trait IbcCallbackEndpoint: Handler + ModuleInterface + AbstractRegistryAccess {
    /// Handler for the `ExecuteMsg::IbcCallback()` variant.
    fn ibc_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: IbcResponseMsg,
    ) -> Result<Response, Self::Error> {
        let vc_query_result = self
            .abstract_registry(deps.as_ref())?
            .query_module(
                ModuleInfo::from_id_latest(IBC_CLIENT).map_err(Into::into)?,
                &deps.querier,
            )
            .map_err(|e| {
                let err: AbstractError = e.into();
                err.into()
            })?;

        let ibc_client = match vc_query_result.reference.unwrap_native();

        if info.sender.ne(&ibc_client) {
            return Err(AbstractSdkError::CallbackNotCalledByIbcClient {
                caller: info.sender,
                client_addr: ibc_client,
                module: self.info().0.to_string(),
            }
            .into());
        };
        let IbcResponseMsg {
            id,
            msg: callback_msg,
            result,
        } = msg;
        let maybe_handler = self.maybe_ibc_callback_handler(&id);
        maybe_handler.map_or_else(
            || Ok(Response::new()),
            |handler| handler(deps, env, info, self, id, callback_msg, result),
        )
    }
}
