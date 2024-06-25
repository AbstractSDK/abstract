use abstract_standalone::sdk::{AbstractResponse, AbstractSdkError, IbcInterface};
use cosmwasm_std::{to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, StdResult};

use crate::{
    msg::{
        ConfigResponse, CountResponse, MyStandaloneExecuteMsg, MyStandaloneInstantiateMsg,
        MyStandaloneMigrateMsg, MyStandaloneQueryMsg,
    },
    state::{Config, CONFIG, COUNT},
    MyStandalone, MyStandaloneResult, MY_STANDALONE, MY_STANDALONE_ID,
};

const INSTANTIATE_REPLY_ID: u64 = 0;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: MyStandaloneInstantiateMsg,
) -> MyStandaloneResult {
    let config: Config = Config {};
    CONFIG.save(deps.storage, &config)?;
    COUNT.save(deps.storage, &msg.count)?;

    // Init standalone as module
    let is_migratable = true;
    MY_STANDALONE.instantiate(deps.branch(), info, msg.base, is_migratable)?;

    Ok(MY_STANDALONE.response("init"))
}

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
            // Parse msg.callback here!
            Ok(MY_STANDALONE
                .response("test_ibc")
                .set_data(msg.callback.msg))
        }
        MyStandaloneExecuteMsg::ModuleIbc(_msg) => {
            todo!()
        }
    }
}

/// Update the configuration of the standalone
fn update_config(deps: DepsMut, info: MessageInfo, standalone: MyStandalone) -> MyStandaloneResult {
    MY_STANDALONE
        .admin
        .assert_admin(deps.as_ref(), &info.sender)?;
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
    MY_STANDALONE
        .admin
        .assert_admin(deps.as_ref(), &info.sender)?;
    COUNT.save(deps.storage, &count)?;

    Ok(standalone.response("reset"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: MyStandaloneQueryMsg) -> StdResult<Binary> {
    let _standalone = &MY_STANDALONE;
    match msg {
        MyStandaloneQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        MyStandaloneQueryMsg::Count {} => to_json_binary(&query_count(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

fn query_count(deps: Deps) -> StdResult<CountResponse> {
    let count = COUNT.load(deps.storage)?;
    Ok(CountResponse { count })
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(_deps: DepsMut, _env: Env, msg: Reply) -> MyStandaloneResult {
    match msg.id {
        self::INSTANTIATE_REPLY_ID => Ok(crate::MY_STANDALONE.response("instantiate_reply")),
        _ => todo!(),
    }
}

/// Handle the standalone migrate msg
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MyStandaloneMigrateMsg) -> MyStandaloneResult {
    // The Abstract Standalone object does version checking and
    MY_STANDALONE.migrate(deps)?;
    Ok(MY_STANDALONE.response("migrate"))
}
