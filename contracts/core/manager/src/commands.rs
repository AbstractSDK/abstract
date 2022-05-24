use abstract_os::core::manager::queries::query_module_version;
use abstract_os::core::modules::{Module, ModuleInfo, ModuleKind};
use abstract_os::core::proxy::msg::ExecuteMsg as TreasuryMsg;
use abstract_os::native::version_control::msg::CodeIdResponse;
use abstract_os::native::version_control::msg::QueryMsg as VersionQuery;
use abstract_os::native::version_control::state::MODULE_CODE_IDS;
use cosmwasm_std::{
    to_binary, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest, Response,
    StdResult, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, ContractVersion};
use semver::Version;

use crate::contract::ManagerResult;
use crate::error::ManagerError;
use crate::state::*;
use abstract_os::native::module_factory::msg::ExecuteMsg as ModuleFactoryMsg;
use abstract_os::registery::{MANAGER, PROXY};

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
    let proxy_addr = OS_MODULES.load(deps.storage, PROXY)?;

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
            kind: ModuleKind::API,
            ..
        } => {
            response = response.add_message(whitelist_dapp_on_proxy(
                deps.as_ref(),
                proxy_addr.into_string(),
                module_address,
            )?)
        }
        _dapp @ Module {
            kind: ModuleKind::AddOn,
            ..
        } => {
            response = response.add_message(whitelist_dapp_on_proxy(
                deps.as_ref(),
                proxy_addr.into_string(),
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

pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module_name: String) -> ManagerResult {
    // Only root can remove modules
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;

    OS_MODULES.remove(deps.storage, &module_name);

    Ok(Response::new().add_attribute("Removed module", &module_name))
}

pub fn set_admin(deps: DepsMut, info: MessageInfo, admin: String) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    ADMIN.execute_update_admin::<Empty>(deps, info, Some(admin_addr))?;
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

// migrates the module to a new version
pub fn migrate_module(
    deps: DepsMut,
    env: Env,
    module_info: ModuleInfo,
    migrate_msg: Binary,
) -> ManagerResult {
    // Check if trying to upgrade this contract.
    if module_info.name == MANAGER {
        return upgrade_self(deps, env, module_info, migrate_msg);
    }

    let module_addr = if module_info.name == MANAGER {
        env.contract.address
    } else {
        OS_MODULES.load(deps.storage, &module_info.name)?
    };

    let contract = query_module_version(&deps.as_ref(), module_addr.clone())?;
    let new_code_id = get_code_id(deps.as_ref(), module_info, contract)?;

    let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: module_addr.into_string(),
        new_code_id,
        msg: migrate_msg,
    });
    Ok(Response::new().add_message(migration_msg))
}

pub fn update_os_status(deps: DepsMut, info: MessageInfo, new_status: Subscribed) -> ManagerResult {
    let config = CONFIG.load(deps.storage)?;

    if info.sender != config.subscription_address {
        Err(ManagerError::CallerNotSubscriptionContract {})
    } else {
        STATUS.save(deps.storage, &new_status)?;
        Ok(Response::new().add_attribute("new_status", new_status.to_string()))
    }
}

fn get_code_id(
    deps: Deps,
    module_info: ModuleInfo,
    contract: ContractVersion,
) -> Result<u64, ManagerError> {
    let new_code_id: u64;
    let config = CONFIG.load(deps.storage)?;
    match module_info.version {
        Some(new_version) => {
            if new_version.parse::<Version>()? > contract.version.parse::<Version>()? {
                new_code_id = MODULE_CODE_IDS
                    .query(
                        &deps.querier,
                        config.version_control_address,
                        (&module_info.name, &new_version),
                    )?
                    .unwrap();
            } else {
                return Err(ManagerError::OlderVersion(new_version, contract.version));
            };
        }
        None => {
            // Query latest version of contract
            let resp: CodeIdResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.version_control_address.to_string(),
                    msg: to_binary(&VersionQuery::QueryCodeId {
                        module: module_info,
                    })?,
                }))?;
            new_code_id = resp.code_id.u64();
        }
    }
    Ok(new_code_id)
}

fn upgrade_self(
    deps: DepsMut,
    env: Env,
    module_info: ModuleInfo,
    migrate_msg: Binary,
) -> ManagerResult {
    let contract = get_contract_version(deps.storage)?;
    let new_code_id = get_code_id(deps.as_ref(), module_info, contract)?;

    let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: env.contract.address.into_string(),
        new_code_id,
        msg: migrate_msg,
    });
    Ok(Response::new().add_message(migration_msg))
}

pub fn whitelist_dapp_on_proxy(
    _deps: Deps,
    proxy_address: String,
    dapp_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address,
        msg: to_binary(&TreasuryMsg::AddDApp { dapp: dapp_address })?,
        funds: vec![],
    }))
}
