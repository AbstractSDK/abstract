use abstract_os::{
    api::{ApiExecuteMsg, ApiQueryMsg, QueryTradersResponse},
    manager::state::{OsInfo, Subscribed, ADMIN, CONFIG, INFO, OS_MODULES, ROOT, STATUS},
    module_factory::ExecuteMsg as ModuleFactoryMsg,
    objects::module::{Module, ModuleInfo, ModuleKind},
    proxy::ExecuteMsg as TreasuryMsg,
    version_control::{
        state::{API_ADDRESSES, MODULE_CODE_IDS},
        QueryApiAddressResponse, QueryCodeIdResponse, QueryMsg as VersionQuery,
    },
};
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest,
    Response, StdResult, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, ContractVersion};
use semver::Version;

use crate::{contract::ManagerResult, error::ManagerError, validators::validate_name_or_gov_type};
use abstract_os::{MANAGER, PROXY};
use abstract_sdk::{configure_api, manager::query_module_version};

/// Adds, updates or removes provided addresses.
/// Should only be called by contract that adds/removes modules.
/// Factory is admin on init
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

    // Check if module is already enabled.
    if OS_MODULES
        .may_load(deps.storage, &module.info.name)?
        .is_some()
    {
        return Err(ManagerError::ModuleAlreadyAdded {});
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

pub fn exec_on_module(
    deps: DepsMut,
    msg_info: MessageInfo,
    module_name: String,
    exec_msg: Binary,
) -> ManagerResult {
    // Only root can update module configs
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let module_addr = OS_MODULES.load(deps.storage, &module_name)?;

    let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: module_addr.into(),
        msg: exec_msg,
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

pub fn set_admin_and_gov_type(
    deps: DepsMut,
    info: MessageInfo,
    admin: String,
    governance_type: Option<String>,
) -> ManagerResult {
    ADMIN.assert_admin(deps.as_ref(), &info.sender)?;

    let admin_addr = deps.api.addr_validate(&admin)?;
    let previous_admin = ADMIN.get(deps.as_ref())?.unwrap();
    if let Some(new_gov_type) = governance_type {
        let mut info = INFO.load(deps.storage)?;
        validate_name_or_gov_type(&new_gov_type)?;
        info.governance_type = new_gov_type;
        INFO.save(deps.storage, &info)?;
    }

    ADMIN.execute_update_admin::<Empty, Empty>(deps, info, Some(admin_addr))?;
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

/// Replaces the current API with a different version
/// Also moves all the trader permissions to the new contract and removes them from the old
pub fn replace_api(deps: DepsMut, module_info: ModuleInfo) -> ManagerResult {
    let config = CONFIG.load(deps.storage)?;
    let mut msgs = vec![];

    // Makes sure we already have the API installed
    let old_api_addr = OS_MODULES.load(deps.storage, &module_info.name)?;
    let proxy_addr = OS_MODULES.load(deps.storage, PROXY)?;
    let traders: QueryTradersResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: config.version_control_address.to_string(),
            msg: to_binary(&ApiQueryMsg::Traders {
                proxy_address: proxy_addr.to_string(),
            })?,
        }))?;
    // Get the address of the new API
    let new_api_addr = get_api_addr(deps.as_ref(), module_info)?;
    let traders_to_migrate: Vec<String> = traders
        .traders
        .into_iter()
        .map(|addr| addr.into_string())
        .collect();
    // Remove traders from old
    msgs.push(configure_api(
        old_api_addr.to_string(),
        ApiExecuteMsg::UpdateTraders {
            to_add: None,
            to_remove: Some(traders_to_migrate.clone()),
        },
    )?);
    // Remove api as trader on dependencies
    msgs.push(configure_api(
        old_api_addr.to_string(),
        ApiExecuteMsg::Remove {},
    )?);
    // Add traders to new
    msgs.push(configure_api(
        new_api_addr.clone(),
        ApiExecuteMsg::UpdateTraders {
            to_add: Some(traders_to_migrate),
            to_remove: None,
        },
    )?);
    // Remove API permissions from proxy
    msgs.push(remove_dapp_from_proxy(
        deps.as_ref(),
        proxy_addr.to_string(),
        old_api_addr.into_string(),
    )?);
    // Add new API to proxy
    msgs.push(whitelist_dapp_on_proxy(
        deps.as_ref(),
        proxy_addr.into_string(),
        new_api_addr.into_string(),
    )?);

    Ok(Response::new().add_messages(msgs))
}

/// Update the OS information
pub fn update_info(
    deps: DepsMut,
    info: MessageInfo,
    os_name: Option<String>,
    description: Option<String>,
    link: Option<String>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;
    let mut info: OsInfo = INFO.load(deps.storage)?;
    if let Some(os_name) = os_name {
        // validate address format
        info.name = os_name;
    }
    info.description = description;
    info.link = link;
    INFO.save(deps.storage, &info)?;
    Ok(Response::new())
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
    old_contract: ContractVersion,
) -> Result<u64, ManagerError> {
    let new_code_id: u64;
    let config = CONFIG.load(deps.storage)?;
    match module_info.version {
        Some(new_version) => {
            if new_version.parse::<Version>()? > old_contract.version.parse::<Version>()? {
                new_code_id = MODULE_CODE_IDS
                    .query(
                        &deps.querier,
                        config.version_control_address,
                        (&module_info.name, &new_version),
                    )?
                    .unwrap();
            } else {
                return Err(ManagerError::OlderVersion(
                    new_version,
                    old_contract.version,
                ));
            };
        }
        None => {
            // Query latest version of contract
            let resp: QueryCodeIdResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.version_control_address.to_string(),
                    msg: to_binary(&VersionQuery::CodeId {
                        module: module_info,
                    })?,
                }))?;
            new_code_id = resp.code_id.u64();
        }
    }
    Ok(new_code_id)
}

fn get_api_addr(deps: Deps, module_info: ModuleInfo) -> Result<Addr, ManagerError> {
    let config = CONFIG.load(deps.storage)?;
    let new_addr = match module_info.version {
        Some(new_version) => {
            let maybe_new_addr = API_ADDRESSES.query(
                &deps.querier,
                config.version_control_address,
                (&module_info.name, &new_version),
            )?;
            if let Some(new_addr) = maybe_new_addr {
                new_addr
            } else {
                return Err(ManagerError::ApiNotFound(module_info.name, new_version));
            }
        }
        None => {
            // Query latest version of contract
            let resp: QueryApiAddressResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.version_control_address.to_string(),
                    msg: to_binary(&VersionQuery::ApiAddress {
                        module: module_info,
                    })?,
                }))?;
            resp.address
        }
    };
    Ok(new_addr)
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

fn whitelist_dapp_on_proxy(
    _deps: Deps,
    proxy_address: String,
    dapp_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address,
        msg: to_binary(&TreasuryMsg::AddModule {
            module: dapp_address,
        })?,
        funds: vec![],
    }))
}

fn remove_dapp_from_proxy(
    _deps: Deps,
    proxy_address: String,
    dapp_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: proxy_address,
        msg: to_binary(&TreasuryMsg::RemoveModule {
            module: dapp_address,
        })?,
        funds: vec![],
    }))
}
