use crate::{
    account_commands::{receive_balances, receive_dispatch, receive_send_all_back},
    contract::{HostResponse, HostResult},
    error::HostError,
    ibc::{receive_query, receive_register, receive_who_am_i},
    state::{CLIENT_PROXY, CONFIG, PROCESSING_PACKET},
};
use abstract_core::{
    objects::chain_name::ChainName, proxy::state::ADMIN, version_control::AccountBase,
};
use abstract_sdk::{
    base::{ExecuteEndpoint, Handler},
    core::ibc_host::{ExecuteMsg, HostAction, InternalAction, PacketMsg},
    feature_objects::VersionControlContract,
    features::AbstractRegistryAccess,
    AccountVerification, Execution,
};
use cosmwasm_std::{
    from_binary, from_slice, DepsMut, Env, IbcPacketReceiveMsg, IbcReceiveResponse, MessageInfo,
    Response, StdError,
};

pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> HostResult {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            let new_admin = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(new_admin))
                .map_err(Into::into)
        }
        ExecuteMsg::UpdateConfig {
            ans_host_address,
            account_factory_address,
            version_control_address,
        } => update_config(
            deps,
            info,
            ans_host_address,
            version_control_address,
            account_factory_address,
        ),
        ExecuteMsg::RecoverAccount {
            closed_channel,
            account_id,
            msgs,
        } => {
            cw_ownable::assert_owner(deps.storage, &info.sender).unwrap();
            // TODO:
            todo!()
        }
    }
}

/// Updates the host's configuration
fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    ans_host_address: Option<String>,
    version_control_address: Option<String>,
    account_factory_address: Option<String>,
) -> HostResult {
    let mut config = CONFIG.load(deps.storage)?;

    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    if let Some(ans_host_address) = ans_host_address {
        // validate address format
        config.ans_host.address = deps.api.addr_validate(&ans_host_address)?;
    }

    if let Some(version_control_address) = version_control_address {
        // validate address format
        config.version_control = deps.api.addr_validate(&version_control_address)?;
    }

    if let Some(account_factory_address) = account_factory_address {
        // validate address format
        config.account_factory = deps.api.addr_validate(&account_factory_address)?;
    }

    CONFIG.save(deps.storage, &config)?;
    Ok(HostResponse::action("update_config"))
}
