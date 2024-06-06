use crate::{
    contract::{MyStandalone, MyStandaloneResult},
    msg::MyStandaloneExecuteMsg,
    state::{ADMIN, CONFIG, COUNT},
    MY_STANDALONE, MY_STANDALONE_ID,
};

use abstract_standalone::{
    objects::module::ModuleInfo,
    sdk::{AbstractSdkError, IbcInterface, ModuleRegistryInterface},
    std::IBC_CLIENT,
    traits::AbstractResponse,
};
use cosmwasm_std::{DepsMut, Env, MessageInfo};

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MyStandaloneExecuteMsg,
) -> MyStandaloneResult {
    let standalone = MY_STANDALONE;
    match msg {
        MyStandaloneExecuteMsg::UpdateConfig {} => update_config(deps, info, standalone),
        MyStandaloneExecuteMsg::Increment {} => increment(deps, standalone),
        MyStandaloneExecuteMsg::Reset { count } => reset(deps, info, count, standalone),
        MyStandaloneExecuteMsg::IbcCallback(msg) => {
            let ibc_client = MY_STANDALONE.ibc_client(deps.as_ref());

            let ibc_client_addr = ibc_client.module_address()?;
            if info.sender.ne(&ibc_client_addr) {
                return Err(AbstractSdkError::CallbackNotCalledByIbcClient {
                    caller: info.sender,
                    client_addr: ibc_client_addr,
                    module: MY_STANDALONE_ID.to_owned(),
                }
                .into());
            };
            // Parse callbacks here!
            match msg.id {
                _ => Ok(MY_STANDALONE.response("todo")),
            }
        }
        MyStandaloneExecuteMsg::ModuleIbc(_) => todo!(),
    }
}

/// Update the configuration of the app
fn update_config(deps: DepsMut, info: MessageInfo, standalone: MyStandalone) -> MyStandaloneResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut _config = CONFIG.load(deps.storage)?;

    Ok(standalone.response("update_config"))
}

fn increment(deps: DepsMut, standalone: MyStandalone) -> MyStandaloneResult {
    COUNT.update(deps.storage, |count| MyStandaloneResult::Ok(count + 1))?;

    Ok(standalone.response("increment"))
}

fn reset(
    deps: DepsMut,
    info: MessageInfo,
    count: i32,
    standalone: MyStandalone,
) -> MyStandaloneResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    COUNT.save(deps.storage, &count)?;

    Ok(standalone.response("reset"))
}
