use std::str::FromStr;

use abstract_core::{
    ibc::CallbackInfo,
    ibc_client::{
        state::{IbcInfrastructure, IBC_INFRA, REVERSE_POLYTONE_NOTE},
        IbcClientCallback, InstalledModuleIdentification,
    },
    ibc_host,
    manager::{self, ModuleInstallConfig},
    objects::{chain_name::ChainName, module_reference::ModuleReference, AccountId, AssetEntry},
    version_control::AccountBase,
};
use abstract_sdk::{
    core::{
        ibc_client::state::{ACCOUNTS, CONFIG},
        ibc_host::{HostAction, InternalAction},
        objects::{ans_host::AnsHost, version_control::VersionControlContract, ChannelEntry},
        ICS20,
    },
    features::AccountIdentification,
    Resolve,
};
use cosmwasm_std::{
    ensure_eq, to_json_binary, wasm_execute, Binary, Coin, CosmosMsg, Deps, DepsMut, Empty, Env,
    IbcMsg, MessageInfo, Storage,
};
use polytone::callbacks::CallbackRequest;

use crate::{
    contract::{IbcClientResponse, IbcClientResult},
    error::IbcClientError,
};

/// Packet lifetime in seconds
pub const PACKET_LIFETIME: u64 = 60 * 60;

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_ans_host: Option<String>,
    new_version_control: Option<String>,
) -> IbcClientResult {
    // auth check
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    if let Some(ans_host) = new_ans_host {
        cfg.ans_host = AnsHost {
            address: deps.api.addr_validate(&ans_host)?,
        };
    }
    if let Some(version_control) = new_version_control {
        cfg.version_control =
            VersionControlContract::new(deps.api.addr_validate(&version_control)?);
        // New version control address implies new accounts.
        clear_accounts(deps.storage);
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(IbcClientResponse::action("update_config"))
}

/// Registers a chain to the client.
/// This registration includes the counterparty information (note and proxy address)
pub fn execute_register_infrastructure(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    host: String,
    note: String,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
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
        &polytone_note::msg::ExecuteMsg::Execute {
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
    host_chain: String,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
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
    account: AccountBase,
    host_chain: ChainName,
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
        &polytone_note::msg::ExecuteMsg::Execute {
            msgs: vec![wasm_execute(
                // The note's remote proxy will call the ibc host
                remote_ibc_host,
                &ibc_host::ExecuteMsg::Execute {
                    // TODO: consider removing this field
                    proxy_address: account.proxy.to_string(),
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
    _env: Env,
    info: MessageInfo,
    host_chain: String,
    action: HostAction,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;

    let cfg = CONFIG.load(deps.storage)?;

    // The packet we need to send depends on the action we want to execute

    let note_message = match &action {
        HostAction::Dispatch { .. } | HostAction::Helpers(_) => {
            // Verify that the sender is a proxy contract
            let account_base = cfg
                .version_control
                .assert_proxy(&info.sender, &deps.querier)?;

            // get account_id
            let account_id = account_base.account_id(deps.as_ref())?;

            send_remote_host_action(
                deps.as_ref(),
                account_id,
                account_base,
                host_chain,
                action,
                None,
            )?
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
pub fn execute_send_module_to_module_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    source_module: InstalledModuleIdentification,
    target_module: InstalledModuleIdentification,
    msg: Binary,
    callback_info: Option<CallbackInfo>,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    let cfg = CONFIG.load(deps.storage)?;

    // We make sure the source_module is indeed who they say they are
    let registry = cfg
        .version_control
        .query_module(source_module.module_info.clone(), &deps.querier)?;

    match registry.reference {
        ModuleReference::AccountBase(_) => return Err(IbcClientError::Unauthorized {}),
        ModuleReference::Native(_) => return Err(IbcClientError::Unauthorized {}),
        ModuleReference::Adapter(addr) => {
            // For adapters, we just need to verify they have the right address
            if addr != info.sender {
                return Err(IbcClientError::ForbiddenModuleCall {});
            }
        }
        ModuleReference::App(code_id) | ModuleReference::Standalone(code_id) => {
            // We verify the caller has indeed the right code id
            let sender_code_id = deps
                .querier
                .query_wasm_contract_info(info.sender.clone())?
                .code_id;
            if code_id != sender_code_id {
                return Err(IbcClientError::ForbiddenModuleCall {});
            }
            // If it does have the right code id, we verify the specified account has the app installed
            let account_base = cfg.version_control.account_base(
                &source_module
                    .account_id
                    .clone()
                    .ok_or(IbcClientError::ForbiddenModuleCall {})?,
                &deps.querier,
            )?;

            let module_info: manager::ModuleAddressesResponse = deps.querier.query_wasm_smart(
                account_base.manager,
                &manager::QueryMsg::ModuleAddresses {
                    ids: vec![source_module.module_info.id()],
                },
            )?;
            ensure_eq!(
                module_info.modules[0].1,
                info.sender,
                IbcClientError::ForbiddenModuleCall {}
            );
        }
        _ => unimplemented!(
            "This module type didn't exist when implementing module-to-module interactions"
        ),
    }

    // We send a message to the target module on the remote chain
    // Send this message via the Polytone implementation

    let callback_request = callback_info.map(|c| CallbackRequest {
        receiver: env.contract.address.to_string(),
        msg: to_json_binary(&IbcClientCallback::ModuleRemoteAction {
            sender_module: source_module.clone(),
            callback_info: c,
        })
        .unwrap(),
    });
    let ibc_infra = IBC_INFRA.load(deps.storage, &host_chain)?;
    let note_contract = ibc_infra.polytone_note;
    let remote_ibc_host = ibc_infra.remote_abstract_host;

    // message that will be called on the local note contract
    let note_message = wasm_execute(
        note_contract.to_string(),
        &polytone_note::msg::ExecuteMsg::Execute {
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

/// Registers an Abstract Account on a remote chain.
pub fn execute_register_account(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    host_chain: String,
    base_asset: Option<AssetEntry>,
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    let cfg = CONFIG.load(deps.storage)?;

    // Verify that the sender is a proxy contract
    let account_base = cfg
        .version_control
        .assert_proxy(&info.sender, &deps.querier)?;

    // get account_id
    let account_id = account_base.account_id(deps.as_ref())?;
    // get auxiliary information

    let account_info: manager::InfoResponse = deps
        .querier
        .query_wasm_smart(account_base.manager.clone(), &manager::QueryMsg::Info {})?;
    let account_info = account_info.info;

    let note_message = send_remote_host_action(
        deps.as_ref(),
        account_id.clone(),
        account_base,
        host_chain,
        HostAction::Internal(InternalAction::Register {
            description: account_info.description,
            link: account_info.link,
            name: account_info.name,
            base_asset,
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
    host_chain: String,
    funds: Vec<Coin>,
) -> IbcClientResult {
    let host_chain = ChainName::from_str(&host_chain)?;
    let cfg = CONFIG.load(deps.storage)?;
    let ans = cfg.ans_host;
    // Verify that the sender is a proxy contract

    let account_base = cfg
        .version_control
        .assert_proxy(&info.sender, &deps.querier)?;

    // get account_id of Account
    let account_id = account_base.account_id(deps.as_ref())?;
    // load remote account
    let remote_addr = ACCOUNTS.load(
        deps.storage,
        (account_id.trace(), account_id.seq(), &host_chain),
    )?;

    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &ans)?;

    let mut transfers: Vec<CosmosMsg> = vec![];
    for amount in funds {
        // construct a packet to send

        transfers.push(
            IbcMsg::Transfer {
                channel_id: ics20_channel_id.clone(),
                to_address: remote_addr.clone(),
                amount,
                timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
            }
            .into(),
        );
    }

    Ok(IbcClientResponse::action("handle_send_funds")
        //.add_message(proxy_msg)
        .add_messages(transfers))
}

fn clear_accounts(store: &mut dyn Storage) {
    ACCOUNTS.clear(store);
}
