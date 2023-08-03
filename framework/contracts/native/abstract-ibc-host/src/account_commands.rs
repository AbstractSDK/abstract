use crate::{
    endpoints::reply::RECEIVE_DISPATCH_ID,
    ibc::PACKET_LIFETIME,
    state::{CONFIG, RESULTS},
    HostError,
};
use abstract_core::{
    manager,
    objects::{chain_name::ChainName, AccountId},
    proxy,
    version_control::AccountBase,
    PROXY,
};
use abstract_sdk::{
    core::{
        abstract_ica::{BalancesResponse, DispatchResponse, SendAllBackResponse, StdAck},
        objects::ChannelEntry,
        ICS20,
    },
    feature_objects::VersionControlContract,
    AbstractSdkError, AccountVerification, Resolve,
};
use cosmwasm_std::{
    to_binary, wasm_execute, CosmosMsg, Deps, DepsMut, Env, IbcMsg, IbcReceiveResponse, SubMsg,
};

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
    deps: DepsMut,
    account: AccountBase,
    manager_msg: manager::ExecuteMsg,
) -> Result<IbcReceiveResponse, HostError> {
    // let them know we're fine
    let response = DispatchResponse { results: vec![] };
    let acknowledgement = StdAck::success(response);
    // execute the message on the manager
    let manager_call_msg = wasm_execute(account.manager, &manager_msg, vec![])?;

    // we wrap it in a submessage to properly report results
    let msg = SubMsg::reply_always(manager_call_msg, RECEIVE_DISPATCH_ID);

    // reset the data field
    RESULTS.save(deps.storage, &vec![])?;

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_submessage(msg)
        .add_attribute("action", "receive_dispatch"))
}

/// processes PacketMsg::SendAllBack variant
pub fn receive_send_all_back(
    deps: DepsMut,
    env: Env,
    account: AccountBase,
    client_proxy_address: String,
    client_chain: ChainName,
) -> Result<IbcReceiveResponse, HostError> {
    // let them know we're fine
    let response = SendAllBackResponse {};
    let acknowledgement = StdAck::success(response);

    let wasm_msg = send_all_back(
        deps.as_ref(),
        env,
        account,
        client_proxy_address,
        client_chain,
    )?;
    // reset the data field
    RESULTS.save(deps.storage, &vec![])?;

    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_message(wasm_msg)
        .add_attribute("action", "receive_dispatch"))
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
