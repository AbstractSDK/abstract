use abstract_standalone::sdk::{AbstractResponse, AbstractSdkError, IbcInterface};
use cosmwasm_std::{
    to_json_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, Reply, StdResult,
};
use cw_ica_controller::{
    helpers::{CwIcaControllerCode, CwIcaControllerContract},
    types::{
        callbacks::IcaControllerCallbackMsg,
        msg::options::ChannelOpenInitOptions,
        state::{ChannelState, ChannelStatus},
    },
};

use crate::{
    msg::{
        ConfigResponse, ICACountResponse, MyStandaloneExecuteMsg, MyStandaloneInstantiateMsg,
        MyStandaloneMigrateMsg, MyStandaloneQueryMsg,
    },
    state::{
        Config, IcaContractState, IcaState, CONFIG, CONTRACT_ADDR_TO_ICA_ID, ICA_COUNT, ICA_STATES,
    },
    MyStandalone, MyStandaloneResult, MY_STANDALONE, MY_STANDALONE_ID,
};

const INSTANTIATE_REPLY_ID: u64 = 0;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MyStandaloneInstantiateMsg,
) -> MyStandaloneResult {
    let config: Config = Config {
        ica_controller_code_id: msg.ica_controller_code_id,
    };
    CONFIG.save(deps.storage, &config)?;
    ICA_COUNT.save(deps.storage, &0)?;

    // Init standalone as module
    let is_migratable = true;
    MY_STANDALONE.instantiate(deps.branch(), &env, info, msg.base, is_migratable)?;

    Ok(MY_STANDALONE.response("init"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: MyStandaloneExecuteMsg,
) -> MyStandaloneResult {
    let standalone = MY_STANDALONE;
    match msg {
        MyStandaloneExecuteMsg::CreateIcaContract {
            salt,
            channel_open_init_options,
        } => create_ica_contract(deps, env, info, standalone, salt, channel_open_init_options),
        MyStandaloneExecuteMsg::ReceiveIcaCallback(callback_msg) => {
            ica_callback(deps, info, standalone, callback_msg)
        }
        MyStandaloneExecuteMsg::SendAction { ica_id, msg } => todo!(),
    }
}

fn create_ica_contract(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    standalone: MyStandalone,
    salt: Option<String>,
    channel_open_init_options: ChannelOpenInitOptions,
) -> MyStandaloneResult {
    standalone.admin.assert_admin(deps.as_ref(), &info.sender)?;
    let config = CONFIG.load(deps.storage)?;

    let ica_code = CwIcaControllerCode::new(config.ica_controller_code_id);

    let instantiate_msg = cw_ica_controller::types::msg::InstantiateMsg {
        owner: Some(env.contract.address.to_string()),
        channel_open_init_options,
        send_callbacks_to: Some(env.contract.address.to_string()),
    };

    let ica_count = ICA_COUNT.load(deps.storage)?;

    let salt = salt.unwrap_or(env.block.time.seconds().to_string());
    let label = format!("icacontroller-{}-{}", env.contract.address, ica_count);

    let (cosmos_msg, contract_addr) = ica_code.instantiate2(
        deps.api,
        &deps.querier,
        &env,
        instantiate_msg,
        label,
        Some(env.contract.address.to_string()),
        salt,
    )?;

    let initial_state = IcaContractState::new(contract_addr.clone());

    ICA_STATES.save(deps.storage, ica_count, &initial_state)?;

    CONTRACT_ADDR_TO_ICA_ID.save(deps.storage, contract_addr, &ica_count)?;

    ICA_COUNT.save(deps.storage, &(ica_count + 1))?;

    Ok(standalone
        .response("create_ica_contract")
        .add_message(cosmos_msg))
}

/// Handles ICA controller callback messages.
pub fn ica_callback(
    deps: DepsMut,
    info: MessageInfo,
    standalone: MyStandalone,
    callback_msg: IcaControllerCallbackMsg,
) -> MyStandaloneResult {
    let ica_id = CONTRACT_ADDR_TO_ICA_ID.load(deps.storage, info.sender)?;
    let mut ica_state = ICA_STATES.load(deps.storage, ica_id)?;

    if let IcaControllerCallbackMsg::OnChannelOpenAckCallback {
        channel,
        ica_address,
        tx_encoding,
    } = callback_msg
    {
        ica_state.ica_state = Some(IcaState {
            ica_id,
            channel_state: ChannelState {
                channel,
                channel_status: ChannelStatus::Open,
            },
            ica_addr: ica_address,
            tx_encoding,
        });

        ICA_STATES.save(deps.storage, ica_id, &ica_state)?;
    }

    Ok(standalone.response("ica_callback"))
}

/// Sends a predefined action to the ICA host.
pub fn send_action(
    deps: DepsMut,
    info: MessageInfo,
    standalone: MyStandalone,
    ica_id: u64,
    msg: CosmosMsg,
) -> MyStandaloneResult {
    standalone.admin.assert_admin(deps.as_ref(), &info.sender)?;

    let ica_state = ICA_STATES.load(deps.storage, ica_id)?;

    let cw_ica_contract = CwIcaControllerContract::new(Addr::unchecked(ica_state.contract_addr));

    let ica_controller_msg = cw_ica_controller::types::msg::ExecuteMsg::SendCosmosMsgs {
        messages: vec![msg],
        packet_memo: None,
        timeout_seconds: None,
    };

    let msg = cw_ica_contract.call(ica_controller_msg)?;

    Ok(standalone.response("send_action").add_message(msg))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: MyStandaloneQueryMsg) -> StdResult<Binary> {
    let _standalone = &MY_STANDALONE;
    match msg {
        MyStandaloneQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        MyStandaloneQueryMsg::ICACount {} => to_json_binary(&query_ica_count(deps)?),
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let _config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {})
}

fn query_ica_count(deps: Deps) -> StdResult<ICACountResponse> {
    let count = ICA_COUNT.load(deps.storage)?;
    Ok(ICACountResponse { count })
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
