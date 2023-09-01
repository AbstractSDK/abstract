use crate::{
    contract::{HostResponse, HostResult},
    endpoints::reply::RESPONSE_REPLY_ID,
    HostError,
};
use abstract_core::{
    ibc_host::state::CONFIG,
    manager,
    objects::{chain_name::ChainName, AccountId},
    proxy,
    version_control::AccountBase,
    PROXY,
};
use abstract_sdk::{
    core::{
        abstract_ica::{BalancesResponse, SendAllBackResponse, StdAck},
        objects::ChannelEntry,
        ICS20,
    },
    feature_objects::VersionControlContract,
    AbstractSdkError, AccountVerification, Resolve,
};
use cosmwasm_std::{
    to_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcReceiveResponse, Response,
    SubMsg,
};

// one hour
const PACKET_LIFETIME: u64 = 60 * 60;

pub fn receive_balances(
    deps: DepsMut,
    account: AccountBase,
) -> Result<IbcReceiveResponse, HostError> {
    let balances = deps.querier.query_all_balances(&account.proxy)?;
    let response = BalancesResponse {
        account: account.proxy.into(),
        balances,
    };
    let acknowledgement = StdAck::success(response);
    // and we are golden
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "receive_balances"))
}

/// Execute manager message on local manager.
pub fn receive_dispatch(
    _deps: DepsMut,
    account: AccountBase,
    manager_msg: manager::ExecuteMsg,
) -> HostResult {
    // execute the message on the manager
    let manager_call_msg = wasm_execute(account.manager, &manager_msg, vec![])?;

    // We want to forward the data that this execution gets
    let submsg = SubMsg::reply_on_success(manager_call_msg, RESPONSE_REPLY_ID);

    // Polytone handles all the necessary
    Ok(Response::new()
        .add_submessage(submsg)
        .add_attribute("action", "receive_dispatch"))
}

/// processes PacketMsg::SendAllBack variant
pub fn receive_send_all_back(
    deps: DepsMut,
    env: Env,
    account: AccountBase,
    client_proxy_address: String,
    client_chain: ChainName,
) -> HostResult {
    // let them know we're fine
    let response = SendAllBackResponse {};
    let _acknowledgement = StdAck::success(response);

    let wasm_msg = send_all_back(
        deps.as_ref(),
        env,
        account,
        client_proxy_address,
        client_chain,
    )?;

    Ok(HostResponse::action("receive_dispatch").add_message(wasm_msg))
}

/// construct the msg to send all the assets back
pub fn send_all_back(
    deps: Deps,
    env: Env,
    account: AccountBase,
    client_proxy_address: String,
    client_chain: ChainName,
) -> Result<CosmosMsg, HostError> {
    // get the ICS20 channel information
    let ans = CONFIG.load(deps.storage)?.ans_host;
    let ics20_channel_entry = ChannelEntry {
        connected_chain: client_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;
    // get all the coins for the account
    let coins = deps.querier.query_all_balances(account.proxy)?;
    // Construct ics20 messages to send all the coins back
    let mut msgs: Vec<CosmosMsg> = vec![];
    for coin in coins {
        msgs.push(
            IbcMsg::Transfer {
                channel_id: ics20_channel_id.clone(),
                to_address: client_proxy_address.to_string(),
                amount: coin,
                timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
            }
            .into(),
        )
    }
    // call the message to send everything back through the manager
    let manager_msg = wasm_execute(
        account.manager,
        &manager::ExecuteMsg::ExecOnModule {
            module_id: PROXY.into(),
            exec_msg: to_binary(&proxy::ExecuteMsg::ModuleAction { msgs })?,
        },
        vec![],
    )?;
    Ok(manager_msg.into())
}

/// get the account base from the version control contract
pub fn get_account(deps: Deps, account_id: &AccountId) -> Result<AccountBase, AbstractSdkError> {
    let version_control = VersionControlContract::new(CONFIG.load(deps.storage)?.version_control);
    version_control
        .account_registry(deps)
        .account_base(account_id)
}
