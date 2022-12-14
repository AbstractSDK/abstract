use crate::{error::ExtensionError, state::ExtensionContract, ExtensionResult};
use abstract_sdk::{
    base::{
        endpoints::{ExecuteEndpoint, IbcCallbackEndpoint, ReceiveEndpoint},
        Handler,
    },
    os::extension::{BaseExecuteMsg, ExecuteMsg},
    ApplicationInterface, Execution, Verification,
};
use cosmwasm_std::{
    to_binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Response, StdError, WasmMsg,
};
use schemars::JsonSchema;
use serde::Serialize;

impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg: Serialize + JsonSchema,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg: Serialize + JsonSchema,
    > ExecuteEndpoint
    for ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    type ExecuteMsg = ExecuteMsg<CustomExecMsg, ReceiveMsg>;

    fn execute(
        mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        msg: Self::ExecuteMsg,
    ) -> Result<Response, Error> {
        let sender = &info.sender;
        match msg {
            ExecuteMsg::App(request) => {
                let core = match request.proxy_address {
                    Some(addr) => {
                        let proxy_addr = deps.api.addr_validate(&addr)?;
                        let traders = self.traders.load(deps.storage, proxy_addr)?;
                        if traders.contains(sender) {
                            self.os_register(deps.as_ref())
                                .assert_proxy(&deps.api.addr_validate(&addr)?)?
                        } else {
                            self.os_register(deps.as_ref())
                                .assert_manager(sender)
                                .map_err(
                                    |_| ExtensionError::UnauthorizedTraderExtensionRequest {},
                                )?
                        }
                    }
                    None => self
                        .os_register(deps.as_ref())
                        .assert_manager(sender)
                        .map_err(|_| ExtensionError::UnauthorizedTraderExtensionRequest {})?,
                };
                self.target_os = Some(core);
                self.execute_handler()?(deps, env, info, self, request.request)
            }
            ExecuteMsg::Base(exec_msg) => self
                .base_execute(deps, env, info.clone(), exec_msg)
                .map_err(From::from),
            ExecuteMsg::IbcCallback(msg) => self.handle_ibc_callback(deps, env, info, msg),
            ExecuteMsg::Receive(msg) => self.handle_receive(deps, env, info, msg),
            #[allow(unreachable_patterns)]
            _ => Err(StdError::generic_err("Unsupported extension execute message variant").into()),
        }
    }
}

/// The extension-contract base implementation.
impl<
        Error: From<cosmwasm_std::StdError> + From<ExtensionError>,
        CustomExecMsg,
        CustomInitMsg,
        CustomQueryMsg,
        ReceiveMsg,
    > ExtensionContract<Error, CustomExecMsg, CustomInitMsg, CustomQueryMsg, ReceiveMsg>
{
    fn base_execute(
        &mut self,
        deps: DepsMut,
        env: Env,
        info: MessageInfo,
        message: BaseExecuteMsg,
    ) -> ExtensionResult {
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
    ) -> Result<Response, ExtensionError> {
        let core = self
            .os_register(deps)
            .assert_manager(&info.sender)
            .map_err(|_| ExtensionError::UnauthorizedExtensionRequest {})?;
        self.target_os = Some(core);
        let dependencies = self.dependencies();
        let mut msgs: Vec<CosmosMsg> = vec![];
        let applications = self.applications(deps);
        for dep in dependencies {
            let extension_addr = applications.app_address(dep.id);
            // just skip if dep is already removed. This means all the traders are already removed.
            if extension_addr.is_err() {
                continue;
            };
            msgs.push(CosmosMsg::Wasm(WasmMsg::Execute {
                contract_addr: extension_addr?.into_string(),
                msg: to_binary(&BaseExecuteMsg::UpdateTraders {
                    to_add: None,
                    to_remove: Some(vec![env.contract.address.to_string()]),
                })?,
                funds: vec![],
            }));
        }
        self.executor(deps)
            .execute_response(msgs, "remove extension from dependencies")
            .map_err(Into::into)
    }

    // pub(crate) fn verify_sender_is_manager(
    //     &self,
    //     deps: Deps,
    //     maybe_manager: &Addr,
    // ) -> Result<Core, ExtensionError> {
    //     let version_control_addr = self.base_state.load(deps.storage)?.version_control;
    //     let core = verify_os_manager(&deps.querier, maybe_manager, &version_control_addr)?;
    //     Ok(core)
    // }

    // pub(crate) fn verify_sender_is_proxy(
    //     &self,
    //     deps: Deps,
    //     maybe_proxy: &Addr,
    // ) -> Result<Core, ExtensionError> {
    //     let version_control_addr = self.base_state.load(deps.storage)?.version_control;
    //     let core = verify_os_proxy(&deps.querier, maybe_proxy, &version_control_addr)?;
    //     Ok(core)
    // }

    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ExtensionError> {
        // Either manager or proxy can add/remove traders.
        // This allows other extensions to automatically add themselves, allowing for extension-cross-calling.
        let core = self
            .os_register(deps.as_ref())
            .assert_manager(&info.sender)?;

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
                    return Err(ExtensionError::TraderAlreadyPresent { trader });
                }
            }
        }

        // Handling the removal of traders
        if let Some(to_remove) = to_remove {
            for trader in to_remove {
                let trader_addr = deps.api.addr_validate(trader.as_str())?;
                if !traders.remove(&trader_addr) {
                    return Err(ExtensionError::TraderNotPresent { trader });
                }
            }
        }

        self.traders.save(deps.storage, proxy.clone(), &traders)?;
        Ok(Response::new().add_attribute("action", format!("update_{}_traders", proxy)))
    }
}
