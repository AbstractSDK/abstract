use cosmwasm_std::{Deps, DepsMut, MessageInfo, Response, StdResult};

use crate::memory::item::Memory;
use crate::treasury::dapp_base::common::BaseDAppResult;
use crate::treasury::dapp_base::msg::{BaseExecuteMsg, BaseInstantiateMsg};
use crate::treasury::dapp_base::state::{ADMIN, BASESTATE};

use super::state::BaseState;

/// Handles the common base execute messages
pub fn handle_base_message(
    deps: DepsMut,
    info: MessageInfo,
    message: BaseExecuteMsg,
) -> BaseDAppResult {
    match message {
        BaseExecuteMsg::UpdateConfig {
            treasury_address,
            trader,
            memory,
        } => update_config(deps, info, treasury_address, trader, memory),
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
        treasury_address: deps.api.addr_validate(&msg.treasury_address)?,
        trader: deps.api.addr_validate(&msg.trader)?,
        memory,
    };

    Ok(state)
}

//----------------------------------------------------------------------------------------
//  GOVERNANCE CONTROLLED SETTERS
//----------------------------------------------------------------------------------------

/// Updates trader or treasury address
pub fn update_config(
    deps: DepsMut,
    info: MessageInfo,
    treasury_address: Option<String>,
    trader: Option<String>,
    memory: Option<String>,
) -> BaseDAppResult {
    // Only the admin should be able to call this
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let mut state = BASESTATE.load(deps.storage)?;

    if let Some(treasury_address) = treasury_address {
        state.treasury_address = deps.api.addr_validate(treasury_address.as_str())?;
    }

    if let Some(trader) = trader {
        state.trader = deps.api.addr_validate(trader.as_str())?;
    }

    if let Some(memory) = memory {
        state.memory.address = deps.api.addr_validate(memory.as_str())?;
    }

    BASESTATE.save(deps.storage, &state)?;
    Ok(Response::new().add_attribute("Update:", "Successful"))
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
