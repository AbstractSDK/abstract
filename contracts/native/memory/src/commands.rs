use abstract_os::objects::{UncheckedChannelEntry, UncheckedContractEntry};
use cosmwasm_std::Env;
use cosmwasm_std::{Addr, DepsMut, Empty, MessageInfo, Response, StdResult};
use cw_asset::{AssetInfo, AssetInfoUnchecked};

use crate::contract::MemoryResult;
use abstract_os::memory::state::*;
use abstract_os::memory::ExecuteMsg;

/// Handles the common base execute messages
pub fn handle_message(
    deps: DepsMut,
    info: MessageInfo,
    _env: Env,
    message: ExecuteMsg,
) -> MemoryResult {
    match message {
        ExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
        ExecuteMsg::UpdateContractAddresses { to_add, to_remove } => {
            update_contract_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateAssetAddresses { to_add, to_remove } => {
            update_asset_addresses(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateChannels { to_add, to_remove } => {
            update_channels(deps, info, to_add, to_remove)
        }
    }
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Adds, updates or removes provided addresses.
pub fn update_contract_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedContractEntry, String)>,
    to_remove: Vec<UncheckedContractEntry>,
) -> MemoryResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_address) in to_add.into_iter() {
        let key = key.check();
        // validate addr
        // let addr = deps.as_ref().api.addr_validate(&new_address)?;
        // Update function for new or existing keys
        let insert = |_| -> StdResult<Addr> { Ok(Addr::unchecked(new_address)) };
        CONTRACT_ADDRESSES.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CONTRACT_ADDRESSES.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_asset_addresses(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(String, AssetInfoUnchecked)>,
    to_remove: Vec<String>,
) -> MemoryResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (name, new_asset) in to_add.into_iter() {
        // Update function for new or existing keys
        let api = deps.api;
        let insert = |_| -> StdResult<AssetInfo> {
            // use own check, cw_asset otherwise changes cases to lowercase
            new_asset.check(api, None)
        };
        ASSET_ADDRESSES.update(deps.storage, name.into(), insert)?;
    }

    for name in to_remove {
        ASSET_ADDRESSES.remove(deps.storage, name.into());
    }

    Ok(Response::new().add_attribute("action", "updated asset addresses"))
}

/// Adds, updates or removes provided addresses.
pub fn update_channels(
    deps: DepsMut,
    msg_info: MessageInfo,
    to_add: Vec<(UncheckedChannelEntry, String)>,
    to_remove: Vec<UncheckedChannelEntry>,
) -> MemoryResult {
    // Only Admin can call this method
    ADMIN.assert_admin(deps.as_ref(), &msg_info.sender)?;

    for (key, new_channel) in to_add.into_iter() {
        let key = key.check();
        // Update function for new or existing keys
        let insert = |_| -> StdResult<String> { Ok(new_channel) };
        CHANNELS.update(deps.storage, key, insert)?;
    }

    for key in to_remove {
        let key = key.check();
        CHANNELS.remove(deps.storage, key);
    }

    Ok(Response::new().add_attribute("action", "updated contract addresses"))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> MemoryResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
