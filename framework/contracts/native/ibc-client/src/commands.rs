use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    features::AccountIdentification,
    namespaces::BASE_STATE,
    HookMemoBuilder, ModuleRegistryInterface, Resolve,
};
use abstract_std::{
    account::{self, ModuleInstallConfig},
    app::AppState,
    ibc::{polytone_callbacks::CallbackRequest, Callback, ModuleQuery},
    ibc_client::{
        state::{
            AccountCallbackPayload, IbcInfrastructure, ACCOUNTS, IBC_INFRA,
            ICS20_ACCOUNT_CALLBACK_PAYLOAD, REVERSE_POLYTONE_NOTE,
        },
        IbcClientCallback, InstalledModuleIdentification, PolytoneNoteExecuteMsg,
    },
    ibc_host::{self, HostAction, InternalAction},
    objects::{
        module::ModuleInfo, module_reference::ModuleReference, AccountId, ChannelEntry,
        TruncatedChainId,
    },
    registry::Account,
    IBC_CLIENT, ICS20,
};
use cosmwasm_std::{
    ensure, to_json_binary, wasm_execute, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env,
    IbcMsg, MessageInfo, QueryRequest, SubMsg, WasmQuery,
};
use cw_storage_plus::Item;

use crate::{
    contract::{IbcClientResponse, IbcClientResult, SEND_FUNDS_WITH_ACTIONS_REPLY_ID},
    error::IbcClientError,
};

/// Packet lifetime in seconds
pub const PACKET_LIFETIME: u64 = 60 * 60;

/// Registers a chain to the client.
/// This registration includes the counterparty information (note and proxy address)
pub fn execute_register_infrastructure(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: TruncatedChainId,
    host: String,
    note: String,
) -> IbcClientResult {
    host_chain.verify()?;

    // auth check
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let note = deps.api.addr_validate(&note)?;
    // Can't allow if it already exists
    if IBC_INFRA.has(deps.storage, &host_chain) || REVERSE_POLYTONE_NOTE.has(deps.storage, &note) {
        return Err(IbcClientError::HostAddressExists {});
    }

    IBC_INFRA.save(
        deps.storage,
        &host_chain,
        &IbcInfrastructure {
            polytone_note: note.clone(),
            remote_abstract_host: host,
            remote_proxy: None,
        },
    )?;
    REVERSE_POLYTONE_NOTE.save(deps.storage, &note, &host_chain)?;

    // When registering a new chain host, we need to get the remote proxy address of the local note.
    // We do so by calling an empty message on the polytone note. This will come back in form of a execute by callback

    let note_proxy_msg = wasm_execute(
        note,
        &PolytoneNoteExecuteMsg::Execute {
            msgs: vec![],
            callback: Some(CallbackRequest {
                receiver: env.contract.address.to_string(),
                msg: to_json_binary(&IbcClientCallback::WhoAmI {})?,
            }),
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;

    Ok(IbcClientResponse::action("allow_chain_port").add_message(note_proxy_msg))
}

// allows admins to clear host if needed
pub fn execute_remove_host(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: TruncatedChainId,
) -> IbcClientResult {
    host_chain.verify()?;

    // auth check
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    if let Some(ibc_infra) = IBC_INFRA.may_load(deps.storage, &host_chain)? {
        REVERSE_POLYTONE_NOTE.remove(deps.storage, &ibc_infra.polytone_note);
    }
    IBC_INFRA.remove(deps.storage, &host_chain);

    Ok(IbcClientResponse::action("remove_host"))
}

/// Send a message to a remote abstract-ibc-host. This message will be proxied through polytone.
fn send_remote_host_action(
    deps: Deps,
    account_id: AccountId,
    account: Account,
    host_chain: TruncatedChainId,
    action: HostAction,
    callback_request: Option<CallbackRequest>,
) -> IbcClientResult<CosmosMsg<Empty>> {
    // Send this message via the Polytone implementation
    let ibc_infra = IBC_INFRA.load(deps.storage, &host_chain)?;
    let note_contract = ibc_infra.polytone_note;
    let remote_ibc_host = ibc_infra.remote_abstract_host;

    // message that will be called on the local note contract
    let note_message = wasm_execute(
        note_contract.to_string(),
        &PolytoneNoteExecuteMsg::Execute {
            msgs: vec![wasm_execute(
                // The note's remote proxy will call the ibc host
                remote_ibc_host,
                &ibc_host::ExecuteMsg::Execute {
                    account_address: account.addr().to_string(),
                    account_id,
                    action,
                },
                vec![],
            )?
            .into()],
            callback: callback_request,
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;

    Ok(note_message.into())
}

/// Sends a packet with an optional callback.
/// This is the top-level function to do IBC related actions.
pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: TruncatedChainId,
    action: HostAction,
) -> IbcClientResult {
    host_chain.verify()?;

    let registry = RegistryContract::new(deps.api, &env)?;
    // The packet we need to send depends on the action we want to execute

    let note_message = match &action {
        HostAction::Dispatch { .. } | HostAction::Helpers(_) => {
            // Verify that the sender is a account contract
            let account = registry.assert_account(&info.sender, &deps.querier)?;

            // get account_id
            let account_id = account.account_id(deps.as_ref())?;

            send_remote_host_action(deps.as_ref(), account_id, account, host_chain, action, None)?
        }
        HostAction::Internal(_) => {
            // Can only call non-internal actions
            return Err(IbcClientError::ForbiddenInternalCall {});
        }
    };

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(note_message))
}

/// Sends a packet with an optional callback.
/// This is the top-level function to do IBC related actions.
#[allow(clippy::too_many_arguments)]
pub fn execute_send_module_to_module_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: TruncatedChainId,
    target_module: ModuleInfo,
    msg: Binary,
    callback: Option<Callback>,
) -> IbcClientResult {
    host_chain.verify()?;

    let registry = RegistryContract::new(deps.api, &env)?;

    // Query the sender module information
    let module_info = registry
        .module_registry(deps.as_ref(), &env)?
        .module_info(info.sender.clone())?;

    // We need additional information depending on the module type
    let source_module = match module_info.reference {
        ModuleReference::Account(_)
        | ModuleReference::Native(_)
        | ModuleReference::Standalone(_)
        | ModuleReference::Service(_) => return Err(IbcClientError::Unauthorized {}),
        ModuleReference::Adapter(_) => InstalledModuleIdentification {
            module_info: module_info.info,
            account_id: None,
        },
        ModuleReference::App(_) => {
            // We verify the associated account id
            let account = Item::<AppState>::new(BASE_STATE)
                .query(&deps.querier, info.sender.clone())?
                .account;
            let account_id = registry.account_id(account.addr(), &deps.querier)?;
            let account = registry.account(&account_id, &deps.querier)?;
            let ibc_client = account::state::ACCOUNT_MODULES.query(
                &deps.querier,
                account.into_addr(),
                IBC_CLIENT,
            )?;
            // Check that ibc_client is installed on account
            ensure!(
                ibc_client.is_some(),
                IbcClientError::IbcClientNotInstalled {
                    account_id: account_id.clone()
                }
            );

            InstalledModuleIdentification {
                module_info: module_info.info,
                account_id: Some(account_id),
            }
        }
        _ => unimplemented!(
            "This module type didn't exist when implementing module-to-module interactions"
        ),
    };

    // We send a message to the target module on the remote chain
    // Send this message via the Polytone implementation

    let callback_request = callback.map(|c| CallbackRequest {
        receiver: env.contract.address.to_string(),
        msg: to_json_binary(&IbcClientCallback::ModuleRemoteAction {
            sender_address: info.sender.to_string(),
            callback: c,
            initiator_msg: msg.clone(),
        })
        .unwrap(),
    });
    let ibc_infra = IBC_INFRA.load(deps.storage, &host_chain)?;
    let note_contract = ibc_infra.polytone_note;
    let remote_ibc_host = ibc_infra.remote_abstract_host;

    // message that will be called on the local note contract
    let note_message = wasm_execute(
        note_contract.to_string(),
        &PolytoneNoteExecuteMsg::Execute {
            msgs: vec![wasm_execute(
                // The note's remote proxy will call the ibc host
                remote_ibc_host,
                &ibc_host::ExecuteMsg::ModuleExecute {
                    msg,
                    source_module,
                    target_module,
                },
                vec![],
            )?
            .into()],
            callback: callback_request,
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;
    Ok(IbcClientResponse::action("handle_send_module_to_module_packet").add_message(note_message))
}

/// Sends a packet with an optional callback.
/// This is the top-level function to do IBC related actions.
pub fn execute_send_query(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: TruncatedChainId,
    queries: Vec<QueryRequest<ModuleQuery>>,
    callback: Callback,
) -> IbcClientResult {
    host_chain.verify()?;
    let ibc_infra = IBC_INFRA.load(deps.storage, &host_chain)?;

    let callback_msg = &IbcClientCallback::ModuleRemoteQuery {
        callback,
        sender_address: info.sender.to_string(),
        // We send un-mapped queries here to enable easily mapping to them.
        queries: queries.clone(),
    };

    let callback_request = CallbackRequest {
        receiver: env.contract.address.to_string(),
        msg: to_json_binary(&callback_msg).unwrap(),
    };

    // Convert custom query type to executable queries
    let queries: Vec<QueryRequest<Empty>> = queries
        .into_iter()
        .map(|q| map_query(&ibc_infra.remote_abstract_host, q))
        .collect();

    let note_contract = ibc_infra.polytone_note;
    let note_message = wasm_execute(
        note_contract.to_string(),
        &PolytoneNoteExecuteMsg::Query {
            msgs: queries,
            callback: callback_request,
            timeout_seconds: PACKET_LIFETIME.into(),
        },
        vec![],
    )?;

    Ok(IbcClientResponse::action("handle_send_msgs").add_message(note_message))
}

/// Registers an Abstract Account on a remote chain.
pub fn execute_register_account(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    host_chain: TruncatedChainId,
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
) -> IbcClientResult {
    host_chain.verify()?;
    let registry = RegistryContract::new(deps.api, &env)?;

    // Verify that the sender is a account contract
    let account = registry.assert_account(&info.sender, &deps.querier)?;

    // get account_id
    let account_id = account.account_id(deps.as_ref())?;
    // get auxiliary information

    let account_info: account::InfoResponse = deps
        .querier
        .query_wasm_smart(account.addr(), &account::QueryMsg::Info {})?;
    let account_info = account_info.info;

    let note_message = send_remote_host_action(
        deps.as_ref(),
        account_id.clone(),
        account,
        host_chain,
        HostAction::Internal(InternalAction::Register {
            description: account_info.description,
            link: account_info.link,
            name: account_info.name,
            namespace,
            install_modules,
        }),
        Some(CallbackRequest {
            receiver: env.contract.address.to_string(),
            msg: to_json_binary(&IbcClientCallback::CreateAccount { account_id })?,
        }),
    )?;

    Ok(IbcClientResponse::action("handle_register").add_message(note_message))
}

pub fn execute_send_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: TruncatedChainId,
    memo: Option<String>,
    receiver: Option<String>,
) -> IbcClientResult {
    host_chain.verify()?;

    let registry = RegistryContract::new(deps.api, &env)?;
    let ans = AnsHost::new(deps.api, &env)?;
    // Verify that the sender is a account contract

    let account = registry.assert_account(&info.sender, &deps.querier)?;

    let remote_addr = match receiver {
        Some(addr) => addr,
        None => {
            // get account_id of Account
            let account_id = account.account_id(deps.as_ref())?;
            // load remote account
            ACCOUNTS.load(
                deps.storage,
                (account_id.trace(), account_id.seq(), &host_chain),
            )?
        }
    };

    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;

    let mut transfers: Vec<CosmosMsg> = vec![];
    for coin in info.funds {
        // construct a packet to send
        let ics_20_send = _ics_20_send_msg(
            &env,
            ics20_channel_id.clone(),
            coin,
            remote_addr.clone(),
            memo.clone(),
        );
        transfers.push(ics_20_send);
    }

    Ok(IbcClientResponse::action("handle_send_funds").add_messages(transfers))
}

pub(crate) fn execute_send_funds_with_actions(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: TruncatedChainId,
    actions: Vec<Binary>,
) -> IbcClientResult {
    host_chain.verify()?;
    let coin = cw_utils::one_coin(&info)?;

    let ibc_infra = IBC_INFRA.load(deps.storage, &host_chain)?;
    // Verify that the sender is a account contract
    let registry = RegistryContract::new(deps.api, &env)?;
    let account = registry.assert_account(&info.sender, &deps.querier)?;

    // get account_id of Account
    let account_id = account.account_id(deps.as_ref())?;

    let ans = AnsHost::new(deps.api, &env)?;
    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;

    // Hook for sending the funds correctly to the sender
    let memo = HookMemoBuilder::new(
        ibc_infra.remote_abstract_host.clone(),
        &ibc_host::ExecuteMsg::Fund {
            src_account: account_id,
            src_chain: TruncatedChainId::from_chain_id(&env.block.chain_id),
        },
    )
    .callback(&env)
    .build()?;

    let msg = SubMsg::reply_on_success(
        _ics_20_send_msg(
            &env,
            ics20_channel_id.clone(),
            coin.clone(),
            ibc_infra.remote_abstract_host,
            Some(memo),
        ),
        SEND_FUNDS_WITH_ACTIONS_REPLY_ID,
    );
    // Save payload for use in reply
    ICS20_ACCOUNT_CALLBACK_PAYLOAD.save(
        deps.storage,
        &AccountCallbackPayload {
            channel_id: ics20_channel_id,
            account_address: account.into_addr(),
            funds: coin,
            msgs: actions,
        },
    )?;

    Ok(IbcClientResponse::action("handle_send_funds_with_actions").add_submessage(msg))
}

#[cfg(target_arch = "wasm32")]
fn _ics_20_send_msg(
    env: &Env,
    ics20_channel_id: String,
    coin: Coin,
    receiver: String,
    memo: Option<String>,
) -> CosmosMsg {
    match memo {
        Some(memo) => {
            let value = crate::anybuf::ibc::MsgTransfer {
                source_port: "transfer".to_string(), // ics20 default
                source_channel: ics20_channel_id,
                token: Some(coin.into()),
                sender: env.contract.address.to_string(),
                receiver,
                timeout_height: None,
                timeout_timestamp: env.block.time.plus_seconds(PACKET_LIFETIME).nanos(),
                memo,
            };

            let value = value.to_anybuf().into_vec();
            let value = Binary::from(value);
            #[allow(deprecated)]
            CosmosMsg::Stargate {
                type_url: crate::anybuf::ibc::MsgTransfer::type_url(),
                value,
            }
        }
        None => IbcMsg::Transfer {
            channel_id: ics20_channel_id,
            to_address: receiver,
            amount: coin,
            timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
            memo,
        }
        .into(),
    }
}

// cw-multi-test does not support stargate messages, so we always send ibcmsg in tests
#[cfg(not(target_arch = "wasm32"))]
fn _ics_20_send_msg(
    env: &Env,
    ics20_channel_id: String,
    coin: Coin,
    receiver: String,
    memo: Option<String>,
) -> CosmosMsg {
    IbcMsg::Transfer {
        channel_id: ics20_channel_id,
        to_address: receiver,
        amount: coin,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
        memo,
    }
    .into()
}

// Map a ModuleQuery to a regular query.
fn map_query(ibc_host: &str, query: QueryRequest<ModuleQuery>) -> QueryRequest<Empty> {
    match query {
        QueryRequest::Custom(ModuleQuery { target_module, msg }) => {
            QueryRequest::Wasm(WasmQuery::Smart {
                contract_addr: ibc_host.into(),
                msg: to_json_binary(&ibc_host::QueryMsg::ModuleQuery { target_module, msg })
                    .unwrap(),
            })
        }
        QueryRequest::Bank(query) => QueryRequest::Bank(query),
        QueryRequest::Staking(query) => QueryRequest::Staking(query),
        #[allow(deprecated)]
        QueryRequest::Stargate { path, data } => QueryRequest::Stargate { path, data },
        QueryRequest::Ibc(query) => QueryRequest::Ibc(query),
        QueryRequest::Wasm(query) => QueryRequest::Wasm(query),
        // Distribution flag not enabled on polytone, so should not be accepted.
        // https://github.com/DA0-DA0/polytone/blob/f70440a35f12f97a9018849ca7e6d241a53582ce/Cargo.toml#L30
        // QueryRequest::Distribution(query) => QueryRequest::Distribution(query),
        _ => unimplemented!("Not implemented type of query"),
    }
}
