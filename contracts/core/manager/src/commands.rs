use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, Response, StdResult,
    WasmMsg,
};
use pandora_os::core::modules::{Module, ModuleKind};
use pandora_os::core::treasury::dapp_base::msg::BaseExecuteMsg;
use pandora_os::core::treasury::dapp_base::msg::ExecuteMsg as TemplateExecuteMsg;

use crate::contract::ManagerResult;
use crate::error::ManagerError;
use crate::state::*;
use pandora_os::native::module_factory::msg::ExecuteMsg as ModuleFactoryMsg;
use pandora_os::registery::TREASURY;

pub const DAPP_CREATE_ID: u64 = 1u64;

/// Adds, updates or removes provided addresses.
/// Should only be called by contract that adds/removes modules.
/// Factory is admin on init
/// TODO: Add functionality to version_control (or some other contract) to add and upgrade contracts.
pub fn update_module_addresses(
    deps: DepsMut,
    to_add: Option<Vec<(String, String)>>,
    to_remove: Option<Vec<String>>,
) -> ManagerResult {
    if let Some(modules_to_add) = to_add {
        for (name, new_address) in modules_to_add.into_iter() {
            if name.is_empty() {
                return Err(ManagerError::InvalidModuleName {});
            };
            // validate addr
            OS_MODULES.save(
                deps.storage,
                name.as_str(),
                &deps.api.addr_validate(&new_address)?,
            )?;
        }
    }

    if let Some(modules_to_remove) = to_remove {
        for name in modules_to_remove.into_iter() {
            OS_MODULES.remove(deps.storage, name.as_str());
        }
    }

    Ok(Response::new().add_attribute("action", "update OS module addresses"))
}

// Attempts to create a new module through the Module Factory Contract
pub fn create_module(
    deps: DepsMut,
    msg_info: MessageInfo,
    _env: Env,
    module: Module,
    init_msg: Option<Binary>,
) -> ManagerResult {
    // Only Root can call this method
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Check if dapp is already enabled.
    if OS_MODULES
        .may_load(deps.storage, &module.info.name)?
        .is_some()
    {
        return Err(ManagerError::InternalDappAlreadyAdded {});
    }

    let config = CONFIG.load(deps.storage)?;

    let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.module_factory_address.into(),
        msg: to_binary(&ModuleFactoryMsg::CreateModule { module, init_msg })?,
        funds: vec![],
    }));

    Ok(response)
}

// Sets the Treasury address on the module if applicable and adds it to the state
pub fn register_module(
    mut deps: DepsMut,
    msg_info: MessageInfo,
    _env: Env,
    module: Module,
    module_address: String,
) -> ManagerResult {
    let config = CONFIG.load(deps.storage)?;
    let treasury_addr = OS_MODULES.load(deps.storage, TREASURY)?;

    // check if sender is module factory
    if msg_info.sender != config.module_factory_address {
        return Err(ManagerError::CallerNotFactory {});
    }

    let mut response = update_module_addresses(
        deps.branch(),
        Some(vec![(module.info.name.clone(), module_address.clone())]),
        None,
    )?;

    match module {
        _dapp @ Module {
            kind: ModuleKind::External,
            ..
        }
        | _dapp @ Module {
            kind: ModuleKind::Internal,
            ..
        } => {
            response = response.add_message(set_treasury_on_dapp(
                deps.as_ref(),
                treasury_addr.into_string(),
                module_address,
            )?)
        }
        Module {
            kind: ModuleKind::Service,
            ..
        } => (),
        Module {
            kind: ModuleKind::Perk,
            ..
        } => (),
    };

    Ok(response)
}

pub fn configure_module(
    deps: DepsMut,
    msg_info: MessageInfo,
    module_name: String,
    config_msg: Binary,
) -> ManagerResult {
    // Only root can update module configs
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;

    let module_addr = OS_MODULES.load(deps.storage, &module_name)?;

    let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: module_addr.into(),
        msg: config_msg,
        funds: vec![],
    }));

    Ok(response)
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin(deps, info, Some(admin_addr))?;
    Ok(Response::default()
        .add_attribute("previous admin", previous_admin)
        .add_attribute("admin", admin))
}

// Only owner can execute it
pub fn execute_update_config(
    deps: DepsMut,
    info: MessageInfo,
    version_control_contract: Option<String>,
    root: Option<String>,
) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;
    if let Some(version_control_contract) = version_control_contract {
        config.version_control_address = deps.api.addr_validate(&version_control_contract)?;
        CONFIG.save(deps.storage, &config)?;
    }

    if let Some(root) = root {
        let addr = deps.api.addr_validate(&root)?;
        ROOT.set(deps, Some(addr))?;
    }

    Ok(Response::new().add_attribute("action", "update_config"))
}

pub fn set_treasury_on_dapp(
    _deps: Deps,
    treasury_address: String,
    dapp_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: dapp_address,
        msg: to_binary(&TemplateExecuteMsg::Base(BaseExecuteMsg::UpdateConfig {
            treasury_address: Some(treasury_address),
        }))?,
        funds: vec![],
    }))
}
