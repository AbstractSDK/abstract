use crate::{error::ApiError, state::ApiContract, ApiResult};
use abstract_os::{
    api::{BaseExecuteMsg, ExecuteMsg},
    version_control::Core,
};
use abstract_sdk::{
    proxy::query_os_manager_address, query_module_address, verify_os_manager, verify_os_proxy,
    AbstractExecute, IbcCallbackEndpoint, OsExecute, ReceiveEndpoint,
};
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use serde::{de::DeserializeOwned, Serialize};

impl<
        'a,
        T: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<ApiError>,
        C: Serialize + DeserializeOwned,
    > AbstractExecute for ApiContract<'a, T, E, C>
{
    type RequestMsg = T;

    type ExecuteMsg<P> = ExecuteMsg<T, C>;

    type ContractError = E;

    fn execute(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg<Self::RequestMsg>,
        request_handler: impl FnOnce(DepsMut, Env, MessageInfo, Self, T) -> Result<Response, E>,
    ) -> Result<Response, Self::ContractError> {
        let sender = &info.sender;
        match msg {
            ExecuteMsg::Request(request) => {
                let core = match request.proxy_address {
                    Some(addr) => {
                        let traders = self
                            .traders
                            .load(deps.storage, Addr::unchecked(addr.clone()))?;
                        if traders.contains(sender) {
                            let proxy = Addr::unchecked(addr);
                            let manager = query_os_manager_address(&deps.querier, &proxy)?;
                            Core { manager, proxy }
                        } else {
                            self.verify_sender_is_manager(deps.as_ref(), sender)
                                .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?
                        }
                    }
                    None => self
                        .verify_sender_is_manager(deps.as_ref(), sender)
                        .map_err(|_| ApiError::UnauthorizedApiRequest {})?,
                };
                self.target_os = Some(core);
                request_handler(deps, env, info, self, request.request)
            }
            ExecuteMsg::Configure(exec_msg) => self
                .base_execute(deps, env, info.clone(), exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            ExecuteMsg::Receive(msg) => self.handle_receive(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported API execute message variant").into()),
        }
    }
}

/// The api-contract base implementation.
impl<
        'a,
        T: Serialize + DeserializeOwned,
        C: Serialize + DeserializeOwned,
        E: From<cosmwasm_std::StdError> + From<ApiError>,
    > ApiContract<'a, T, E, C>
{
    fn base_execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> ApiResult {
        match message {
            BaseExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
            BaseExecuteMsg::Remove {} => self.remove_self_from_deps(deps.as_ref(), env, info),
        }
    }

    /// If dependencies are set, remove self from them.
    pub(crate) fn remove_self_from_deps(
        &mut self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ApiError> {
        let core = self.verify_sender_is_manager(deps, &info.sender)?;
        // Dangerous to forget!! add to verify fn?
        self.target_os = Some(core);
        let dependencies = self.dependencies;
        let mut msgs: Vec<CosmosMsg> = vec![];
        for dep in dependencies {
            let api_addr =
                query_module_address(deps, &self.target_os.as_ref().unwrap().manager, dep);
            // just skip if dep is already removed. This means all the traders are already removed.
            if api_addr.is_err() {
                continue;
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api_addr?.into_string(),
                msg: to_binary(&BaseExecuteMsg::UpdateTraders {
                    to_add: None,
                    to_remove: Some(vec![env.contract.address.to_string()]),
                })?,
                funds: vec![],
            }));
        }
        let msg = self.os_execute(deps, msgs)?;
        Ok(Response::new().add_submessage(msg))
    }

    pub(crate) fn verify_sender_is_manager(
        &self,
        deps: Deps,
        maybe_manager: &Addr,
    ) -> Result<Core, ApiError> {
        let version_control_addr = self.base_state.load(deps.storage)?.version_control;
        let core = verify_os_manager(&deps.querier, maybe_manager, &version_control_addr)?;
        Ok(core)
    }

    pub(crate) fn verify_sender_is_proxy(
        &self,
        deps: Deps,
        maybe_proxy: &Addr,
    ) -> Result<Core, ApiError> {
        let version_control_addr = self.base_state.load(deps.storage)?.version_control;
        let core = verify_os_proxy(&deps.querier, maybe_proxy, &version_control_addr)?;
        Ok(core)
    }

    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ApiError> {
        // Either manager or proxy can add/remove traders.
        // This allows other apis to automatically add themselves, allowing for api-cross-calling.
        let core = {
            self.verify_sender_is_manager(deps.as_ref(), &info.sender)
                .or_else(|_| self.verify_sender_is_proxy(deps.as_ref(), &info.sender))
        }?;

        // Manager can only change traders for associated proxy
        let proxy = core.proxy;

        let mut traders = self
            .traders
            .may_load(deps.storage, proxy.clone())?
            .unwrap_or_default();

        // Handle the addition of traders
        if let Some(to_add) = to_add {
            for trader in to_add {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.insert(trader_addr) {
                    return Err(ApiError::TraderAlreadyPresent { trader });
                }
            }
        }

        // Handling the removal of traders
        if let Some(to_remove) = to_remove {
            for trader in to_remove {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.remove(&trader_addr) {
                    return Err(ApiError::TraderNotPresent { trader });
                }
            }
        }

        self.traders.save(deps.storage, proxy.clone(), &traders)?;
        Ok(Response::new().add_attribute("action", format!("update_{}_traders", proxy)))
    }
}
