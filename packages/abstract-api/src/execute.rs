use abstract_os::api::ApiExecuteMsg;
use abstract_sdk::proxy::send_to_proxy;
use abstract_sdk::OsExecute;
use cosmwasm_std::{Deps, DepsMut, Env, MessageInfo, Response};
use serde::de::DeserializeOwned;
use serde::Serialize;

use crate::error::ApiError;
use crate::state::ApiContract;
use crate::ApiResult;

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

impl<'a, T: Serialize + DeserializeOwned> ApiContract<'a, T> {
    pub fn execute(
        &self,
        deps: DepsMut,
        _env: Env,
        info: MessageInfo,
        message: ApiExecuteMsg,
    ) -> ApiResult {
        match message {
            ApiExecuteMsg::UpdateTraders { to_add, to_remove } => {
                self.update_traders(deps, info, to_add, to_remove)
            }
        }
    }

    fn update_traders(
        &self,
        deps: DepsMut,
        info: MessageInfo,
        to_add: Option<Vec<String>>,
        to_remove: Option<Vec<String>>,
    ) -> Result<Response, ApiError> {
        // Only the manager of the proxy can add/remove traders
        let core = self.verify_sender_is_manager(deps.as_ref(), &info.sender)?;

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
        Ok(
            Response::new()
                .add_attribute("action", format!("update_{}_traders", proxy.to_string())),
        )
    }
}
