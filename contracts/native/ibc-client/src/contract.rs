use abstract_os::ibc_host::{HostAction, InternalAction, PacketMsg};
use abstract_os::objects::memory::Memory;
use abstract_os::objects::ChannelEntry;
use abstract_os::{IBC_CLIENT, ICS20};
use abstract_sdk::proxy::query_os_id;
use abstract_sdk::{os_module_action, verify_os_proxy, Resolve};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, Order, QueryResponse,
    Response, StdError, StdResult,
};
use cw2::set_contract_version;

use crate::error::ClientError;
use crate::ibc::PACKET_LIFETIME;
use abstract_os::ibc_client::state::{
    AccountData, Config, ACCOUNTS, CHANNELS, CONFIG, LATEST_QUERIES, MEMORY,
};
use abstract_os::ibc_client::{
    AccountInfo, AccountResponse, CallbackInfo, ConfigResponse, ExecuteMsg, InstantiateMsg,
    LatestQueryResponse, ListAccountsResponse, ListChannelsResponse, MigrateMsg, QueryMsg,
};
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");
const MAX_RETRIES: u8 = 5;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    let cfg = Config {
        admin: info.sender,
        chain: msg.chain,
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
    };
    CONFIG.save(deps.storage, &cfg)?;
    MEMORY.save(
        deps.storage,
        &Memory {
            address: deps.api.addr_validate(&msg.memory_address)?,
        },
    )?;
    set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;

    Ok(Response::new().add_attribute("action", "instantiate"))
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ClientError> {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            execute_update_admin(deps, info, admin).map_err(Into::into)
        }
        ExecuteMsg::SendPacket {
            host_chain,
            action,
            callback_info,
            retries,
        } => execute_send_packet(deps, env, info, host_chain, action, callback_info, retries),
        ExecuteMsg::SendFunds { host_chain, funds } => {
            execute_send_funds(deps, env, info, host_chain, funds).map_err(Into::into)
        }
        ExecuteMsg::Register { host_chain } => execute_register_os(deps, env, info, host_chain),
        ExecuteMsg::RemoveHost { host_chain } => {
            execute_remove_host(deps, info, host_chain).map_err(Into::into)
        }
    }
}

pub fn execute_update_admin(
    deps: DepsMut,
    info: MessageInfo,
    new_admin: String,
) -> StdResult<Response> {
    // auth check
    let mut cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Only admin may set new admin"));
    }
    cfg.admin = deps.api.addr_validate(&new_admin)?;
    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new()
        .add_attribute("action", "handle_update_admin")
        .add_attribute("new_admin", cfg.admin))
}

// allows admins to clear host if needed
pub fn execute_remove_host(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: String,
) -> StdResult<Response> {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    if info.sender != cfg.admin {
        return Err(StdError::generic_err("Only admin may remove hosts"));
    }
    CHANNELS.remove(deps.storage, &host_chain);

    Ok(Response::new()
        .add_attribute("action", "handle_remove_host")
        .add_attribute("new_admin", cfg.admin))
}

pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    mut action: HostAction,
    callback_info: Option<CallbackInfo>,
    mut retries: u8,
) -> Result<Response, ClientError> {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let core = verify_os_proxy(&deps.querier, &info.sender, &cfg.version_control_address)?;
    // Can only call non-internal actions
    if let HostAction::Internal(_) = action {
        return Err(ClientError::ForbiddenInternalCall {});
    }
    // fill proxy address on send-all-back
    if let HostAction::SendAllBack { os_proxy_address } = &mut action {
        *os_proxy_address = Some(core.proxy.into_string())
    };
    // Set max retries
    retries = retries.min(MAX_RETRIES);

    // get os_id
    let os_id = query_os_id(&deps.querier, &core.manager)?;
    // ensure the channel exists and loads it.
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    let packet = PacketMsg {
        retries,
        client_chain: cfg.chain,
        os_id,
        callback_info,
        action,
    };
    let msg = IbcMsg::SendPacket {
        channel_id: channel,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_send_msgs");
    Ok(res)
}

pub fn execute_register_os(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
) -> Result<Response, ClientError> {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let core = verify_os_proxy(&deps.querier, &info.sender, &cfg.version_control_address)?;
    // ensure the channel exists (not found if not registered)
    let channel_id = CHANNELS.load(deps.storage, &host_chain)?;
    let os_id = query_os_id(&deps.querier, &core.manager)?;

    // construct a packet to send
    let packet = PacketMsg {
        retries: 0u8,
        client_chain: cfg.chain,
        os_id,
        callback_info: None,
        action: HostAction::Internal(InternalAction::Register),
    };

    // save a default value to account
    let account = AccountData::default();
    ACCOUNTS.save(deps.storage, (&channel_id, os_id), &account)?;

    let msg = IbcMsg::SendPacket {
        channel_id,
        data: to_binary(&packet)?,
        timeout: env.block.time.plus_seconds(PACKET_LIFETIME).into(),
    };

    let res = Response::new()
        .add_message(msg)
        .add_attribute("action", "handle_register");
    Ok(res)
}

pub fn execute_send_funds(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    funds: Vec<Coin>,
) -> StdResult<Response> {
    let cfg = CONFIG.load(deps.storage)?;
    let mem = MEMORY.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let core = verify_os_proxy(&deps.querier, &info.sender, &cfg.version_control_address)?;
    // get os_id of OS
    let os_id = query_os_id(&deps.querier, &core.manager)?;
    // get channel used to communicate to host chain
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    // load remote account
    let data = ACCOUNTS.load(deps.storage, (&channel, os_id))?;
    let remote_addr = match data.remote_addr {
        Some(addr) => addr,
        None => {
            return Err(StdError::generic_err(
                "We don't have the remote address for this channel or OS",
            ))
        }
    };

    let ics20_channel_entry = ChannelEntry {
        connected_chain: host_chain,
        protocol: ICS20.to_string(),
    };
    let ics20_channel_id = ics20_channel_entry.resolve(deps.as_ref(), &mem)?;

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
    // let these messages be executed by proxy
    let proxy_msg = os_module_action(transfers, &core.proxy)?;
    let res = Response::new()
        .add_message(proxy_msg)
        .add_attribute("action", "handle_send_funds");
    Ok(res)
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Admin {} => to_binary(&query_config(deps)?),
        QueryMsg::Account { chain, os_id } => to_binary(&query_account(deps, chain, os_id)?),
        QueryMsg::ListAccounts {} => to_binary(&query_list_accounts(deps)?),
        QueryMsg::LatestQueryResult { chain, os_id } => {
            to_binary(&query_latest_ibc_query_result(deps, chain, os_id)?)
        }
        QueryMsg::ListChannels {} => to_binary(&query_list_channels(deps)?),
    }
}

fn query_account(deps: Deps, host_chain: String, os_id: u32) -> StdResult<AccountResponse> {
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    let account = ACCOUNTS.load(deps.storage, (&channel, os_id))?;
    Ok(account.into())
}

fn query_latest_ibc_query_result(
    deps: Deps,
    host_chain: String,
    os_id: u32,
) -> StdResult<LatestQueryResponse> {
    let channel = CHANNELS.load(deps.storage, &host_chain)?;
    LATEST_QUERIES.load(deps.storage, (&channel, os_id))
}

// TODO: paging
fn query_list_accounts(deps: Deps) -> StdResult<ListAccountsResponse> {
    let accounts = ACCOUNTS
        .range(deps.storage, None, None, Order::Ascending)
        .map(|r| {
            let ((channel_id, os_id), account) = r?;
            Ok(AccountInfo::convert(channel_id, os_id, account))
        })
        .collect::<StdResult<_>>()?;
    Ok(ListAccountsResponse { accounts })
}

fn query_list_channels(deps: Deps) -> StdResult<ListChannelsResponse> {
    let channels = CHANNELS
        .range(deps.storage, None, None, Order::Ascending)
        .collect::<StdResult<_>>()?;
    Ok(ListChannelsResponse { channels })
}

fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let Config {
        admin,
        chain,
        version_control_address,
    } = CONFIG.load(deps.storage)?;
    Ok(ConfigResponse {
        admin: admin.into(),
        chain,
        version_control_address: version_control_address.into_string(),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;

    // let version: Version = CONTRACT_VERSION.parse().unwrap();
    // let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();
    // if storage_version < version {
    // set_contract_version(deps.storage, OSMOSIS_HOST, CONTRACT_VERSION)?;
    // }
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};

    const CREATOR: &str = "creator";

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            chain: "test_chain".into(),
            memory_address: "memory".into(),
            version_control_address: "vc_addr".into(),
        };
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config = query_config(deps.as_ref()).unwrap();
        assert_eq!(CREATOR, config.admin.as_str());
    }
}
