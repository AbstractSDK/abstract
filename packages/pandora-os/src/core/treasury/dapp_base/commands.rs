use cosmwasm_std::{Addr, Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::core::treasury::dapp_base::common::BaseDAppResult;
use crate::core::treasury::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg};
use crate::core::treasury::dapp_base::state::{ADMIN, BASESTATE};
use crate::native::memory::item::Memory;

use super::error::BaseDAppError;
use super::state::BaseState;

/// Handles the common base execute messages
pub fn handle_base_message(
    deps: DepsMut,
    info: MessageInfo,
    message: BaseExecuteMsg,
) -> BaseDAppResult {
    match message {
        BaseExecuteMsg::UpdateConfig { treasury_address } => {
            update_config(deps, info, treasury_address)
        }
        BaseExecuteMsg::UpdateTraders { to_add, to_remove } => {
            update_traders(deps, info, to_add, to_remove)
        }
        BaseExecuteMsg::SetAdmin { admin } => set_admin(deps, info, admin),
    }
}

/// Handles creates the State and Memory object and returns them.
pub fn handle_base_init(deps: Deps, msg: BaseInstantiateMsg) -> StdResult<BaseState> {
    // Memory
    let memory = Memory {
        address: deps.api.addr_validate(&msg.memory_addr)?,
    };
    // Base state
    let state = BaseState {
        // Treasury gets set by manager after Init
        treasury_address: Addr::unchecked(""),
        traders: vec![],
        memory,
    };

    Ok(state)
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Updates traders or treasury address
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    treasury_address: Option<String>,
) -> BaseDAppResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = BASESTATE.load(deps.storage)?;

    if let Some(treasury_address) = treasury_address {
        state.treasury_address = deps.api.addr_validate(treasury_address.as_str())?;
    }

    BASESTATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("Update:", "Successful"))
}

/// Handles updating traders.
/// Adds are evaluated before removes so if a trader resides in both,
/// it will be removed.
fn update_traders(
    deps: DepsMut,
    info: MessageInfo,
    to_add: Option<Vec<String>>,
    to_remove: Option<Vec<String>>,
) -> BaseDAppResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = BASESTATE.load(deps.storage)?;

    // Handle the addition of traders
    if let Some(to_add) = to_add {
        for trader in to_add {
            let trader_addr = deps.api.addr_validate(trader.as_str())?;
            if !state.traders.contains(&trader_addr) {
                state.traders.push(trader_addr);
            } else {
                return Err(BaseDAppError::TraderAlreadyPresent { trader });
            }
        }
    }

    // Handling the removal of traders
    if let Some(to_remove) = to_remove {
        for trader in to_remove {
            let trader_addr = deps.api.addr_validate(trader.as_str())?;
            if let Some(trader_pos) = state.traders.iter().position(|a| a == &trader_addr) {
                state.traders.remove(trader_pos);
            } else {
                return Err(BaseDAppError::TraderNotPresent { trader });
            }
        }
    }

    // at least one trader is always required
    if state.traders.is_empty() {
        return Err(BaseDAppError::TraderRequired {});
    }

    BASESTATE.save(deps.storage, &state)?;
    // TODO: do we want to return diff somewhat like in CW4 update members spec?
    // https://docs.cosmwasm.com/cw-plus/0.9.0/cw4/spec
    Ok(Response::new().add_attribute("action", "update_traders"))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> BaseDAppResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}
