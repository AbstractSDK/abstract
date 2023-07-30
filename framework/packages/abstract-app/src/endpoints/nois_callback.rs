use cosmwasm_std::{
    DepsMut,
    Env,
    MessageInfo,
    Response,
    ensure_eq
};
use nois::NoisCallback;
use abstract_core::objects::{UncheckedContractEntry};
use abstract_sdk::{
    base::NoisHandler,
    features::AbstractNameService
};
use crate::{AppContract, NoisCallbackEndpoint, AppError};

const NOIS_PROTOCOL: &str = "nois";
const NOIS_PROXY_CONTRACT: &str = "proxy";

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
    fn nois_callback(self, deps: DepsMut, env: Env, info: MessageInfo, callback: NoisCallback) -> Result<Response, Self::Error> {
        let ans_host = self.ans_host(deps.as_ref())?;

        let nois_proxy = ans_host.query_contract(&deps.querier, &UncheckedContractEntry::new(NOIS_PROTOCOL, NOIS_PROXY_CONTRACT).check())?;

        //callback should only be allowed to be called by the proxy contract
        // otherwise anyone can cut the randomness workflow and cheat the randomness by sending the randomness directly to this contract
        ensure_eq!(info.sender, nois_proxy, AppError::UnauthorizedNoisCallback {
            caller: info.sender,
            proxy_addr: nois_proxy
        });

        let Some(handler) = self.maybe_nois_callback_handler() else {
            return Ok(Response::new())
        };
        handler(deps, env, info, self, callback)
    }
}
