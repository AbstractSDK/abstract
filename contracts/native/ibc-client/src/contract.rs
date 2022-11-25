use abstract_os::ibc_client::state::ADMIN;
use abstract_sdk::{
    base::features::Identification,
    feature_objects::VersionControlContract,
    os::{
        ibc_host::{HostAction, InternalAction, PacketMsg},
        objects::{ans_host::AnsHost, ChannelEntry},
        IBC_CLIENT, ICS20,
    },
    Execution, Resolve, Verification,
};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{
    to_binary, Coin, CosmosMsg, Deps, DepsMut, Env, IbcMsg, MessageInfo, Order, QueryResponse,
    Response, StdError, StdResult, Storage,
};
use cw2::set_contract_version;

use crate::{error::ClientError, ibc::PACKET_LIFETIME};
use abstract_sdk::os::ibc_client::{
    state::{AccountData, Config, ACCOUNTS, ANS_HOST, CHANNELS, CONFIG, LATEST_QUERIES},
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
        chain: msg.chain,
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
    };
    CONFIG.save(deps.storage, &cfg)?;
    ANS_HOST.save(
        deps.storage,
        &AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    )?;
    set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;

    ADMIN.set(deps, Some(info.sender))?;
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
            let new_admin = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(new_admin))
                .map_err(Into::into)
        }
        ExecuteMsg::UpdateConfig {
            ans_host,
            version_control,
        } => execute_update_config(deps, info, ans_host, version_control).map_err(Into::into),
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

pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    new_ans_host: Option<String>,
    new_version_control: Option<String>,
) -> Result<Response, ClientError> {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut cfg = CONFIG.load(deps.storage)?;

    if let Some(ans_host) = new_ans_host {
        ANS_HOST.save(
            deps.storage,
            &AnsHost {
                address: deps.api.addr_validate(&ans_host)?,
            },
        )?;
    }
    if let Some(version_control) = new_version_control {
        cfg.version_control_address = deps.api.addr_validate(&version_control)?;
        // New version control address implies new accounts.
        clear_accounts(deps.storage);
    }

    CONFIG.save(deps.storage, &cfg)?;

    Ok(Response::new().add_attribute("action", "update_config"))
}

// allows admins to clear host if needed
pub fn execute_remove_host(
    deps: DepsMut,
    info: MessageInfo,
    host_chain: String,
) -> Result<Response, ClientError> {
    // auth check
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    CHANNELS.remove(deps.storage, &host_chain);

    Ok(Response::new().add_attribute("action", "remove_host"))
}

pub fn execute_send_packet(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    host_chain: String,
    action: HostAction,
    callback_info: Option<CallbackInfo>,
    mut retries: u8,
) -> Result<Response, ClientError> {
    // auth check
    let cfg = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract {
        contract_address: cfg.version_control_address,
    };
    // Verify that the sender is a proxy contract
    let core = version_control
        .os_register(deps.as_ref())
        .assert_proxy(&info.sender)?;
    // Can only call non-internal actions
    if let HostAction::Internal(_) = action {
        return Err(ClientError::ForbiddenInternalCall {});
    }
    // Set max retries
    retries = retries.min(MAX_RETRIES);

    // get os_id
    let os_id = core.os_id(deps.as_ref())?;
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
    let version_control = VersionControlContract {
        contract_address: cfg.version_control_address,
    };
    let core = version_control
        .os_register(deps.as_ref())
        .assert_proxy(&info.sender)?;
    // ensure the channel exists (not found if not registered)
    let channel_id = CHANNELS.load(deps.storage, &host_chain)?;
    let os_id = core.os_id(deps.as_ref())?;

    // construct a packet to send
    let packet = PacketMsg {
        retries: 0u8,
        client_chain: cfg.chain,
        os_id,
        callback_info: None,
        action: HostAction::Internal(InternalAction::Register {
            os_proxy_address: core.proxy.into_string(),
        }),
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
    let mem = ANS_HOST.load(deps.storage)?;
    // Verify that the sender is a proxy contract
    let version_control = VersionControlContract {
        contract_address: cfg.version_control_address,
    };
    let core = version_control
        .os_register(deps.as_ref())
        .assert_proxy(&info.sender)?;
    // get os_id of OS
    let os_id = core.os_id(deps.as_ref())?;
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
    let ics20_channel_id = ics20_channel_entry.resolve(&deps.querier, &mem)?;

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
    let proxy_msg = core.executor(deps.as_ref()).execute(transfers)?;

    let res = Response::new()
        .add_message(proxy_msg)
        .add_attribute("action", "handle_send_funds");
    Ok(res)
}

fn clear_accounts(store: &mut dyn Storage) {
    ACCOUNTS.clear(store);
    LATEST_QUERIES.clear(store);
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
        chain,
        version_control_address,
    } = CONFIG.load(deps.storage)?;
    let admin = ADMIN.get(deps)?.unwrap();
    Ok(ConfigResponse {
        admin: admin.into(),
        chain,
        version_control_address: version_control_address.into_string(),
    })
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    // type migration
    let config = old_abstract_os::ibc_client::state::CONFIG.load(deps.storage)?;
    let new_config = Config {
        chain: config.chain,
        version_control_address: config.version_control_address,
    };
    CONFIG.save(deps.storage, &new_config)?;
    ADMIN.set(deps, Some(config.admin))?;
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
            ans_host_address: "ans_host".into(),
            version_control_address: "vc_addr".into(),
        };
        let info = mock_info(CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_eq!(0, res.messages.len());

        let config = query_config(deps.as_ref()).unwrap();
        assert_eq!(CREATOR, config.admin.as_str());
    }
}
