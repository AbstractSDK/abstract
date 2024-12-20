use abstract_standalone::sdk::AbstractResponse;
use cosmwasm_std::{Addr, Binary, CosmosMsg, Deps, DepsMut, Env, MessageInfo, QueryRequest, Reply, StdResult, to_json_binary};
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
    MY_STANDALONE,
    MyStandalone, MyStandaloneResult, state::{
        Config, CONFIG, CONTRACT_ADDR_TO_ICA_ID, ICA_COUNT, ICA_STATES, IcaContractState, IcaState,
    },
};
use crate::msg::PacketStateResponse;
use crate::state::{EXECUTE_RECEIPTS, increment_sequence_number, QUERY_RECEIPTS};

const INSTANTIATE_REPLY_ID: u64 = 0;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    _env: Env,
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
    MY_STANDALONE.instantiate(deps.branch(), info, msg.base, is_migratable)?;

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
        MyStandaloneExecuteMsg::Execute { ica_id, msgs } => {
            send_action(deps, env, info, standalone, ica_id, msgs)
        }
        MyStandaloneExecuteMsg::Query { ica_id, msgs } => {
            send_queries(deps, env, info, standalone, ica_id, msgs)
        }
        MyStandaloneExecuteMsg::ReceiveIcaCallback(callback_msg) => {
            ica_callback(deps, info, standalone, callback_msg)
        }
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
    standalone
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;
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

    let mut response = standalone.response("ica_callback");

    match callback_msg {
        IcaControllerCallbackMsg::OnChannelOpenAckCallback {
            channel,
            ica_address,
            tx_encoding,
        } =>
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
        IcaControllerCallbackMsg::OnAcknowledgementPacketCallback {
            ica_acknowledgement, original_packet, query_result, ..
        } => {
            EXECUTE_RECEIPTS.save(deps.storage, (ica_id, original_packet.sequence), &ica_acknowledgement)?;
            if let Some(query_result) = query_result {
                // Save the query result
                QUERY_RECEIPTS.save(deps.storage, (ica_id, original_packet.sequence), &query_result)?;
            }

            response = response.add_attribute("sequence", original_packet.sequence.to_string());
        }
        // Do nothing
        _ => ()
    }

    Ok(response)
}

/// Sends a predefined action to the ICA host.
pub fn send_action(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    standalone: MyStandalone,
    ica_id: u64,
    msgs: Vec<CosmosMsg>,
) -> MyStandaloneResult {
    standalone
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;

    let ica_state = ICA_STATES.load(deps.storage, ica_id)?;

    let cw_ica_contract = CwIcaControllerContract::new(Addr::unchecked(ica_state.contract_addr));

    let ica_controller_msg = cw_ica_controller::types::msg::ExecuteMsg::SendCosmosMsgs {
        messages: msgs,
        queries: vec![],
        packet_memo: Some("aloha".to_string()),
        timeout_seconds: None,
    };

    let msg = cw_ica_contract.execute(ica_controller_msg)?;
    let sequence = increment_sequence_number(deps.storage, ica_id)?;

    Ok(standalone.response("send_action").add_message(msg).add_attribute("sequence", sequence.to_string()))
}

/// Sends a predefined action to the ICA host.
pub fn send_queries(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    standalone: MyStandalone,
    ica_id: u64,
    queries: Vec<QueryRequest>,
) -> MyStandaloneResult {
    standalone
        .admin
        .assert_admin(deps.as_ref(), &env, &info.sender)?;

    let ica_state = ICA_STATES.load(deps.storage, ica_id)?;
    let cw_ica_contract = CwIcaControllerContract::new(Addr::unchecked(ica_state.contract_addr));

    let ica_controller_msg = cw_ica_controller::types::msg::ExecuteMsg::SendCosmosMsgs {
        messages: vec![],
        queries,
        packet_memo: Some("queries".to_string()),
        timeout_seconds: None,
    };

    let msg = cw_ica_contract.execute(ica_controller_msg)?;

    let sequence = increment_sequence_number(deps.storage, ica_id)?;

    Ok(standalone.response("send_action").add_message(msg).add_attribute("sequence", sequence.to_string()))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: MyStandaloneQueryMsg) -> StdResult<Binary> {
    let _standalone = &MY_STANDALONE;
    match msg {
        MyStandaloneQueryMsg::Config {} => to_json_binary(&query_config(deps)?),
        MyStandaloneQueryMsg::ICACount {} => to_json_binary(&query_ica_count(deps)?),
        MyStandaloneQueryMsg::IcaContractState { ica_id } => {
            to_json_binary(&ica_state(deps, ica_id)?)
        }
        MyStandaloneQueryMsg::PacketState { ica_id, sequence } => to_json_binary(&query_packet_state(deps, ica_id, sequence)?)
    }
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let config = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        ica_controller_code_id: config.ica_controller_code_id,
    })
}

/// Returns the saved ICA state for the given ICA ID.
pub fn ica_state(deps: Deps, ica_id: u64) -> StdResult<IcaContractState> {
    ICA_STATES.load(deps.storage, ica_id)
}

fn query_ica_count(deps: Deps) -> StdResult<ICACountResponse> {
    let count = ICA_COUNT.load(deps.storage)?;
    Ok(ICACountResponse { count })
}

fn query_packet_state(deps: Deps, ica_id: u64, sequence: u64) -> StdResult<PacketStateResponse> {
    let ack_data = EXECUTE_RECEIPTS.may_load(deps.storage, (ica_id, sequence))?;
    let query_result = QUERY_RECEIPTS.may_load(deps.storage, (ica_id, sequence))?;
    Ok(PacketStateResponse {
        ack_data,
        query_result,
    })
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
