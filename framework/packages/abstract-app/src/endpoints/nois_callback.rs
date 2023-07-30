use crate::{AppContract, AppError, NoisCallbackEndpoint};
use abstract_sdk::{base::NoisHandler, NoisInterface};
use cosmwasm_std::{ensure_eq, DepsMut, Env, MessageInfo, Response};
use nois::NoisCallback;

impl<
        Error: From<cosmwasm_std::StdError>
            + From<AppError>
            + From<abstract_sdk::AbstractSdkError>
            + From<abstract_core::AbstractError>
            + 'static,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    > NoisCallbackEndpoint
    for AppContract<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >
{
    fn nois_callback(
        self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        callback: NoisCallback,
    ) -> Result<Response, Self::Error> {
        let nois_proxy = self.nois_proxy_address(deps.as_ref())?;

        // callback should only be allowed to be called by the proxy contract
        // otherwise anyone can cut the randomness workflow and cheat the randomness by sending the randomness directly to this contract
        ensure_eq!(
            info.sender,
            nois_proxy,
            AppError::UnauthorizedNoisCallback {
                caller: info.sender,
                proxy_addr: nois_proxy
            }
        );

        // Save the randomness to the state
        self.randomness.update(
            deps.storage,
            callback.job_id.clone(),
            |prev| -> Result<_, AppError> {
                match prev {
                    Some(_) => Err(AppError::RandomnessAlreadySet(callback.job_id.to_string())),
                    None => Ok(callback.randomness.clone()),
                }
            },
        )?;

        let Some(handler) = self.maybe_nois_callback_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, callback)
    }
}
