use abstract_os::api::{ApiExecuteMsg, ApiInterfaceMsg};
use abstract_os::version_control::Core;
use abstract_sdk::common_namespace::BASE_STATE_KEY;
use abstract_sdk::manager::query_module_address;
use abstract_sdk::proxy::send_to_proxy;
use abstract_sdk::version_control::{verify_os_manager, verify_os_proxy};
use abstract_sdk::OsExecute;
use cosmwasm_std::{
    to_binary, Addr, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, WasmMsg,
};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ApiError;
use crate::state::{ApiContract, TRADER_NAMESPACE};
use crate::ApiResult;

/// Execute a set of CosmosMsgs on the proxy contract of an OS.
impl<T: Serialize + DeserializeOwned> OsExecute for ApiContract<'_, T> {
    type Err = ApiError;

    fn os_execute(
        &self,
        _deps: Deps,
        msgs: Vec<cosmwasm_std::CosmosMsg>,
    ) -> Result<Response, Self::Err> {
        Ok(Response::new().add_message(send_to_proxy(msgs, &self.request_destination.clone())?))
    }
}

/// The api-contract base implementation.
impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    /// Takes request, sets destination and executes request handler
    /// This fn is the only way to get an ApiContract instance which ensures the destination address is set correctly.
    pub fn handle_request<RequestError: From<cosmwasm_std::StdError> + From<ApiError>>(
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: ApiInterfaceMsg<T>,
        request_handler: impl FnOnce(
            DepsMut,
            Env,
            MessageInfo,
            ApiContract<T>,
            T,
        ) -> Result<Response, RequestError>,
    ) -> Result<Response, RequestError> {
        let sender = &info.sender;
        let mut api = Self::new(BASE_STATE_KEY, TRADER_NAMESPACE, Addr::unchecked(""));
        match msg {
            ApiInterfaceMsg::Request(request) => {
                let proxy = match request.proxy_address {
                    Some(addr) => {
                        let traders = api
                            .traders
                            .load(deps.storage, Addr::unchecked(addr.clone()))?;
                        if traders.contains(sender) {
                            Addr::unchecked(addr)
                        } else {
                            api.verify_sender_is_manager(deps.as_ref(), sender)
                                .map_err(|_| ApiError::UnauthorizedTraderApiRequest {})?
                                .proxy
                        }
                    }
                    None => {
                        api.verify_sender_is_manager(deps.as_ref(), sender)
                            .map_err(|_| ApiError::UnauthorizedApiRequest {})?
                            .proxy
                    }
                };
                api.request_destination = proxy;
                request_handler(deps, env, info, api, request.request)
            }
            ApiInterfaceMsg::Configure(exec_msg) => api
                .execute(deps, env, info.clone(), exec_msg)
                .map_err(From::from),
        }
    }
    pub fn execute(
        &self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: ApiExecuteMsg,
    ) -> ApiResult {
        match message {
            ApiExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
            ApiExecuteMsg::Remove {} => self.remove_self_from_deps(deps.as_ref(), env, info),
        }
    }

    /// If dependencies are set, remove self from them.
    pub(crate) fn remove_self_from_deps(
        &self,
        deps: Deps,
        env: Env,
        info: MessageInfo,
    ) -> Result<Response, ApiError> {
        let core = self.verify_sender_is_manager(deps, &info.sender)?;
        let dependencies = self.state(deps.storage)?.api_dependencies;
        let mut msgs: Vec<CosmosMsg> = vec![];
        for dep in dependencies {
            let api_addr = query_module_address(deps, &core.manager, dep.as_str());
            // just skip if dep is already removed. This means all the traders are already removed.
            if api_addr.is_err() {
                continue;
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: api_addr?.into_string(),
                msg: to_binary(&ApiExecuteMsg::UpdateTraders {
                    to_add: None,
                    to_remove: Some(vec![env.contract.address.to_string()]),
                })?,
                funds: vec![],
            }));
        }
        self.os_execute(deps, msgs)
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
            .load(deps.storage, proxy.clone())
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
