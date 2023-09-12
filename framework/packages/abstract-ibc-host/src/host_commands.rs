use crate::{
    endpoints::reply::INIT_CALLBACK_ID,
    state::{ContractError, CLIENT_PROXY, CLOSED_CHANNELS, PENDING},
    Host, HostError,
};
use abstract_core::objects::AccountId;
use abstract_sdk::core::abstract_ica::{
    check_order, check_version, IbcQueryResponse, StdAck, WhoAmIResponse, IBC_APP_VERSION,
};
use cosmwasm_std::{
    entry_point, to_binary, to_vec, Binary, ContractResult, Deps, DepsMut, Empty, Env, Event,
    Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcChannelOpenResponse, IbcPacketAckMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse, QuerierWrapper, QueryRequest, StdError, StdResult, SubMsg, SystemResult,
    WasmMsg,
};

// one hour
pub const PACKET_LIFETIME: u64 = 60 * 60;

#[entry_point]
#[allow(unused)]
/// enforces ordering and versioing constraints
pub fn ibc_channel_open(
    _deps: DepsMut,
    _env: Env,
    msg: IbcChannelOpenMsg,
) -> Result<IbcChannelOpenResponse, HostError> {
    let channel = msg.channel();

    check_order(&channel.order)?;
    // In ibcv3 we don't check the version string passed in the message
    // and only check the counterparty version.
    if let Some(counter_version) = msg.counterparty_version() {
        check_version(counter_version)?;
    }

    // We return the version we need (which could be different than the counterparty version)
    Ok(Some(Ibc3ChannelOpenResponse {
        version: IBC_APP_VERSION.to_string(),
    }))
}

#[entry_point]
#[allow(unused)]
/// channel established
pub fn ibc_channel_connect(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelConnectMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    let chan_id = &channel.endpoint.channel_id;
    // re-open channel if it was closed previously.
    let re_open_channel = |mut closed_channels: Vec<String>| -> StdResult<Vec<String>> {
        Ok(closed_channels
            .into_iter()
            .filter(|c| c != chan_id)
            .collect())
    };
    CLOSED_CHANNELS.update(deps.storage, re_open_channel)?;

    Ok(IbcBasicResponse::new()
        .add_attribute("action", "ibc_connect")
        .add_attribute("channel_id", chan_id)
        .add_event(Event::new("ibc").add_attribute("channel", "connect")))
}

#[entry_point]
#[allow(unused)]
pub fn ibc_channel_close(
    deps: DepsMut,
    env: Env,
    msg: IbcChannelCloseMsg,
) -> StdResult<IbcBasicResponse> {
    let channel = msg.channel();
    // get contract address and remove lookup
    let channel_id = channel.endpoint.channel_id.clone();
    CLOSED_CHANNELS.update(deps.storage, |mut channels| {
        channels.push(channel_id);
        Ok::<_, StdError>(channels)
    })?;

    Ok(
        IbcBasicResponse::new()
            // .add_submessages(messages)
            .add_attribute("action", "ibc_close")
            .add_attribute("channel_id", channel.endpoint.channel_id.clone()), // .add_attribute("rescue_funds", rescue_funds.to_string())
    )
}

fn unparsed_query(
    querier: QuerierWrapper<'_, Empty>,
    request: &QueryRequest<Empty>,
) -> Result<Binary, HostError> {
    let raw = to_vec(request)?;
    match querier.raw_query(&raw) {
        SystemResult::Err(system_err) => {
            Err(StdError::generic_err(format!("Querier system error: {system_err}")).into())
        }
        SystemResult::Ok(ContractResult::Err(contract_err)) => {
            Err(StdError::generic_err(format!("Querier contract error: {contract_err}")).into())
        }
        SystemResult::Ok(ContractResult::Ok(value)) => Ok(value),
    }
}

// processes IBC query
pub fn receive_query(
    deps: Deps,
    msgs: Vec<QueryRequest<Empty>>,
) -> Result<IbcReceiveResponse, HostError> {
    let mut results = vec![];

    for query in msgs {
        let res = unparsed_query(deps.querier, &query)?;
        results.push(res);
    }
    let response = IbcQueryResponse { results };

    let acknowledgement = StdAck::success(response);
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "receive_ibc_query"))
}

// processes PacketMsg::Register variant
/// Creates and registers proxy for remote Account
pub fn receive_register<
    Error: ContractError,
    CustomExecMsg,
    CustomInitMsg,
    CustomQueryMsg,
    CustomMigrateMsg,
    ReceiveMsg,
    SudoMsg,
>(
    deps: DepsMut,
    env: Env,
    host: Host<
        Error,
        CustomInitMsg,
        CustomExecMsg,
        CustomQueryMsg,
        CustomMigrateMsg,
        ReceiveMsg,
        SudoMsg,
    >,
    channel: String,
    account_id: AccountId,
    account_proxy_address: String,
) -> Result<IbcReceiveResponse, HostError> {
    let cfg = host.base_state.load(deps.storage)?;
    let init_msg = cw1_whitelist::msg::InstantiateMsg {
        admins: vec![env.contract.address.into_string()],
        mutable: false,
    };
    let msg = WasmMsg::Instantiate {
        admin: None,
        code_id: cfg.cw1_code_id,
        msg: to_binary(&init_msg)?,
        funds: vec![],
        label: format!("ibc-reflect-{}", channel.as_str()),
    };
    let msg = SubMsg::reply_on_success(msg, INIT_CALLBACK_ID);

    // store the proxy address of the Account on the client chain.
    CLIENT_PROXY.save(
        deps.storage,
        (&channel, account_id.clone()),
        &account_proxy_address,
    )?;
    // store the account info for the reply handler
    PENDING.save(deps.storage, &(channel, account_id.clone()))?;

    // We rely on Reply handler to change this to Success!
    let acknowledgement = StdAck::fail(format!("Failed to create proxy for Account {account_id} "));

    Ok(IbcReceiveResponse::new()
        .add_submessage(msg)
        .set_ack(acknowledgement)
        .add_attribute("action", "register"))
}

// processes InternalAction::WhoAmI variant
pub fn receive_who_am_i(this_chain: String) -> Result<IbcReceiveResponse, HostError> {
    // let them know we're fine
    let response = WhoAmIResponse { chain: this_chain };
    let acknowledgement = StdAck::success(response);
    Ok(IbcReceiveResponse::new()
        .set_ack(acknowledgement)
        .add_attribute("action", "who_am_i"))
}

#[entry_point]
#[allow(unused)]
/// never should be called as we do not send packets
pub fn ibc_packet_ack(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketAckMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_ack"))
}

#[entry_point]
#[allow(unused)]
/// never should be called as we do not send packets
pub fn ibc_packet_timeout(
    _deps: DepsMut,
    _env: Env,
    _msg: IbcPacketTimeoutMsg,
) -> StdResult<IbcBasicResponse> {
    Ok(IbcBasicResponse::new().add_attribute("action", "ibc_packet_timeout"))
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     use cosmwasm_std::testing::{
//         mock_dependencies, mock_env, mock_ibc_channel_close_init, mock_ibc_channel_connect_ack,
//         mock_ibc_channel_open_init, mock_ibc_channel_open_try, mock_ibc_packet_recv, mock_info,
//         mock_wasmd_attr, MockApi, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
//     };
//     use cosmwasm_std::{
//         attr, coin, coins, from_slice, BankMsg, Binary, OwnedDeps, SubMsgResponse, SubMsgResult,
//         WasmMsg,
//     };
//     use abstract_ica::{APP_ORDER, BAD_APP_ORDER};

//     const CREATOR: &str = "creator";
//     // code id of the reflect contract
//     const REFLECT_ID: u64 = 101;
//     // address of first reflect contract instance that we created
//     const REFLECT_ADDR: &str = "reflect-acct-1";

//     fn setup() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
//         let mut deps = mock_dependencies();
//         let msg = InstantiateMsg {
//             cw1_code_id: REFLECT_ID,
//         };
//         let info = mock_info(CREATOR, &[]);
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len());
//         deps
//     }

//     fn fake_data(reflect_addr: &str) -> Binary {
//         // works with length < 128
//         let mut encoded = vec![0x0a, reflect_addr.len() as u8];
//         encoded.extend(reflect_addr.as_bytes());
//         Binary::from(encoded)
//     }

//     fn fake_events(reflect_addr: &str) -> Vec<Event> {
//         let event = Event::new("instantiate").add_attributes(vec![
//             attr("code_id", "17"),
//             // We have to force this one to avoid the debug assertion against _
//             mock_wasmd_attr("_contract_address", reflect_addr),
//         ]);
//         vec![event]
//     }

//     // connect will run through the entire handshake to set up a proper connect and
//     // save the account (tested in detail in `proper_handshake_flow`)
//     fn connect(mut deps: DepsMut, channel_id: &str, account: impl Into<String>) {
//         let account: String = account.into();

//         let handshake_open = mock_ibc_channel_open_init(channel_id, APP_ORDER, IBC_APP_VERSION);
//         // first we try to open with a valid handshake
//         ibc_channel_open(deps.branch(), mock_env(), handshake_open).unwrap();

//         // then we connect (with counter-party version set)
//         let handshake_connect =
//             mock_ibc_channel_connect_ack(channel_id, APP_ORDER, IBC_APP_VERSION);
//         let res = ibc_channel_connect(deps.branch(), mock_env(), handshake_connect).unwrap();
//         assert_eq!(1, res.messages.len());
//         assert_eq!(1, res.events.len());
//         assert_eq!(
//             Event::new("ibc").add_attribute("channel", "connect"),
//             res.events[0]
//         );
//         let id = res.messages[0].id;

//         // fake a reply and ensure this works
//         let response = Reply {
//             id,
//             result: SubMsgResult::Ok(SubMsgResponse {
//                 events: fake_events(&account),
//                 data: Some(fake_data(&account)),
//             }),
//         };
//         reply(deps.branch(), mock_env(), response).unwrap();
//     }

//     #[test]
//     fn instantiate_works() {
//         let mut deps = mock_dependencies();

//         let msg = InstantiateMsg { cw1_code_id: 17 };
//         let info = mock_info("creator", &[]);
//         let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
//         assert_eq!(0, res.messages.len())
//     }

//     #[test]
//     fn enforce_version_in_handshake() {
//         let mut deps = setup();

//         let wrong_order = mock_ibc_channel_open_try("channel-12", BAD_APP_ORDER, IBC_APP_VERSION);
//         ibc_channel_open(deps.as_mut(), mock_env(), wrong_order).unwrap_err();

//         let wrong_version = mock_ibc_channel_open_try("channel-12", APP_ORDER, "reflect");
//         ibc_channel_open(deps.as_mut(), mock_env(), wrong_version).unwrap_err();

//         let valid_handshake = mock_ibc_channel_open_try("channel-12", APP_ORDER, IBC_APP_VERSION);
//         ibc_channel_open(deps.as_mut(), mock_env(), valid_handshake).unwrap();
//     }

//     #[test]
//     fn proper_handshake_flow() {
//         let mut deps = setup();
//         let channel_id = "channel-1234";

//         // first we try to open with a valid handshake
//         let handshake_open = mock_ibc_channel_open_init(channel_id, APP_ORDER, IBC_APP_VERSION);
//         ibc_channel_open(deps.as_mut(), mock_env(), handshake_open).unwrap();

//         // then we connect (with counter-party version set)
//         let handshake_connect =
//             mock_ibc_channel_connect_ack(channel_id, APP_ORDER, IBC_APP_VERSION);
//         let res = ibc_channel_connect(deps.as_mut(), mock_env(), handshake_connect).unwrap();
//         // and set up a reflect account
//         assert_eq!(1, res.messages.len());
//         let id = res.messages[0].id;
//         if let CosmosMsg::Wasm(WasmMsg::Instantiate {
//             admin,
//             code_id,
//             msg: _,
//             funds,
//             label,
//         }) = &res.messages[0].msg
//         {
//             assert_eq!(*admin, None);
//             assert_eq!(*code_id, REFLECT_ID);
//             assert_eq!(funds.len(), 0);
//             assert!(label.contains(channel_id));
//         } else {
//             panic!("invalid return message: {:?}", res.messages[0]);
//         }

//         // no accounts set yet
//         let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
//         let res: ListAccountsResponse = from_slice(&raw).unwrap();
//         assert_eq!(0, res.accounts.len());

//         // fake a reply and ensure this works
//         let response = Reply {
//             id,
//             result: SubMsgResult::Ok(SubMsgResponse {
//                 events: fake_events(REFLECT_ADDR),
//                 data: Some(fake_data(REFLECT_ADDR)),
//             }),
//         };
//         reply(deps.as_mut(), mock_env(), response).unwrap();

//         // ensure this is now registered
//         let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
//         let res: ListAccountsResponse = from_slice(&raw).unwrap();
//         assert_eq!(1, res.accounts.len());
//         assert_eq!(
//             &res.accounts[0],
//             &AccountInfo {
//                 account: REFLECT_ADDR.into(),
//                 channel_id: channel_id.to_string(),
//             }
//         );

//         // and the account query also works
//         let raw = query(
//             deps.as_ref(),
//             mock_env(),
//             QueryMsg::Account {
//                 channel_id: channel_id.to_string(),
//             },
//         )
//         .unwrap();
//         let res: AccountResponse = from_slice(&raw).unwrap();
//         assert_eq!(res.account.unwrap(), REFLECT_ADDR);
//     }

//     #[test]
//     fn handle_dispatch_packet() {
//         let mut deps = setup();

//         let channel_id = "channel-123";
//         let account = "acct-123";

//         // receive a packet for an unregistered channel returns app-level error (not Result::Err)
//         let msgs_to_dispatch = vec![BankMsg::Send {
//             to_address: "my-friend".into(),
//             amount: coins(123456789, "uatom"),
//         }
//         .into()];
//         let ibc_msg = PacketMsg::Dispatch {
//             msgs: msgs_to_dispatch.clone(),
//             sender: account.to_string(),
//             callback_id: None,
//         };
//         let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
//         // this returns an error
//         ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap_err();

//         // register the channel
//         connect(deps.as_mut(), channel_id, account);

//         // receive a packet for an unregistered channel returns app-level error (not Result::Err)
//         let msg = mock_ibc_packet_recv(channel_id, &ibc_msg).unwrap();
//         let res = ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap();

//         // assert app-level success
//         let ack: StdAck = from_slice(&res.acknowledgement).unwrap();
//         ack.unwrap();

//         // and we dispatch the BankMsg via submessage
//         assert_eq!(1, res.messages.len());
//         assert_eq!(RECEIVE_DISPATCH_ID, res.messages[0].id);

//         // parse the output, ensuring it matches
//         if let CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr,
//             msg,
//             funds,
//         }) = &res.messages[0].msg
//         {
//             assert_eq!(account, contract_addr.as_str());
//             assert_eq!(0, funds.len());
//             // parse the message - should callback with proper channel_id
//             let rmsg: cw1_whitelist::msg::ExecuteMsg = from_slice(msg).unwrap();
//             assert_eq!(
//                 rmsg,
//                 cw1_whitelist::msg::ExecuteMsg::Execute {
//                     msgs: msgs_to_dispatch
//                 }
//             );
//         } else {
//             panic!("invalid return message: {:?}", res.messages[0]);
//         }

//         // invalid packet format on registered channel also returns error
//         let bad_data = InstantiateMsg { cw1_code_id: 12345 };
//         let msg = mock_ibc_packet_recv(channel_id, &bad_data).unwrap();
//         ibc_packet_receive(deps.as_mut(), mock_env(), msg).unwrap_err();
//     }

//     #[test]
//     fn check_close_channel() {
//         let mut deps = setup();

//         let channel_id = "channel-123";
//         let account = "acct-123";

//         // register the channel
//         connect(deps.as_mut(), channel_id, account);
//         // assign it some funds
//         let funds = vec![coin(123456, "uatom"), coin(7654321, "tgrd")];
//         deps.querier.update_balance(account, funds.clone());

//         // channel should be listed and have balance
//         let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
//         let res: ListAccountsResponse = from_slice(&raw).unwrap();
//         assert_eq!(1, res.accounts.len());
//         let balance = deps.as_ref().querier.query_all_balances(account).unwrap();
//         assert_eq!(funds, balance);

//         // close the channel
//         let channel = mock_ibc_channel_close_init(channel_id, APP_ORDER, IBC_APP_VERSION);
//         let res = ibc_channel_close(deps.as_mut(), mock_env(), channel).unwrap();

//         // it pulls out all money from the reflect contract
//         assert_eq!(1, res.messages.len());
//         if let CosmosMsg::Wasm(WasmMsg::Execute {
//             contract_addr, msg, ..
//         }) = &res.messages[0].msg
//         {
//             assert_eq!(contract_addr.as_str(), account);
//             let reflect: ReflectExecuteMsg = from_slice(msg).unwrap();
//             match reflect {
//                 ReflectExecuteMsg::ReflectMsg { msgs } => {
//                     assert_eq!(1, msgs.len());
//                     assert_eq!(
//                         &msgs[0],
//                         &BankMsg::Send {
//                             to_address: MOCK_CONTRACT_ADDR.into(),
//                             amount: funds
//                         }
//                         .into()
//                     )
//                 }
//             }
//         } else {
//             panic!("Unexpected message: {:?}", &res.messages[0]);
//         }

//         // and removes the account lookup
//         let raw = query(deps.as_ref(), mock_env(), QueryMsg::ListAccounts {}).unwrap();
//         let res: ListAccountsResponse = from_slice(&raw).unwrap();
//         assert_eq!(0, res.accounts.len());
//     }
// }
