use cosmwasm_std::{DepsMut, Env, Response};

use crate::{
    contract::{App, AppResult},
    msg::AppSudoMsg,
    state::ICS20_CALLBACKS,
};

pub fn sudo_handler(deps: DepsMut, _env: Env, module: App, msg: AppSudoMsg) -> AppResult {
    match msg {
        AppSudoMsg::IBCLifecycleComplete(ibclifecycle_complete) => {
            let callback = module.load_ics20_callback(deps.storage, &ibclifecycle_complete)?;
            ICS20_CALLBACKS.update(deps.storage, |mut list| {
                list.push((callback, ibclifecycle_complete));
                AppResult::Ok(list)
            })?;
            Ok(Response::new())
        }
    }
}
