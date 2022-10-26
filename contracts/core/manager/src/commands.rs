use abstract_os::{
    api::{BaseExecuteMsg, BaseQueryMsg, QueryMsg as ApiQuery, TradersResponse},
    manager::state::{OsInfo, Subscribed, CONFIG, INFO, OS_MODULES, ROOT, STATUS},
    module_factory::ExecuteMsg as ModuleFactoryMsg,
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
    },
    proxy::ExecuteMsg as TreasuryMsg,
    version_control::{state::MODULE_LIBRARY, ModuleResponse, QueryMsg as VersionQuery},
    IBC_CLIENT,
};
use cosmwasm_std::{
    to_binary, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo, QueryRequest,
    Response, StdError, StdResult, WasmMsg, WasmQuery,
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
    module: ModuleInfo,
    init_msg: Option<Binary>,
) -> ManagerResult {
    // Only Root can call this method
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;

    // Check if module is already enabled.
    if OS_MODULES.may_load(deps.storage, &module.name)?.is_some() {
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
        Some(vec![(module.info.id(), module_address.clone())]),
        None,
    )?;

    match module {
        _dapp @ Module {
            reference: ModuleReference::App(_),
            ..
        } => {
            response = response.add_message(whitelist_dapp_on_proxy(
                deps.as_ref(),
                proxy_addr.into_string(),
                module_address,
            )?)
        }
        _dapp @ Module {
            reference: ModuleReference::Extension(_),
            ..
        } => {
            response = response.add_message(whitelist_dapp_on_proxy(
                deps.as_ref(),
                proxy_addr.into_string(),
                module_address,
            )?)
        }
        _ => (),
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

pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module_id: String) -> ManagerResult {
    // Only root can remove modules
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let proxy = OS_MODULES.load(deps.storage, PROXY)?;
    let module_addr = OS_MODULES.load(deps.storage, &module_id)?;
    let remove_from_proxy_msg = remove_dapp_from_proxy(
        deps.as_ref(),
        proxy.into_string(),
        module_addr.into_string(),
    )?;
    OS_MODULES.remove(deps.storage, &module_id);

    Ok(Response::new()
        .add_message(remove_from_proxy_msg)
        .add_attribute("Removed module", &module_id))
}

pub fn set_root_and_gov_type(
    deps: DepsMut,
    info: MessageInfo,
    root: String,
    governance_type: Option<String>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;

    let root_addr = deps.api.addr_validate(&root)?;
    let previous_root = ROOT.get(deps.as_ref())?.unwrap();
    if let Some(new_gov_type) = governance_type {
        let mut info = INFO.load(deps.storage)?;
        validate_name_or_gov_type(&new_gov_type)?;
        info.governance_type = new_gov_type;
        INFO.save(deps.storage, &info)?;
    }

    ROOT.execute_update_admin::<Empty, Empty>(deps, info, Some(root_addr))?;
    Ok(Response::default()
        .add_attribute("previous root", previous_root)
        .add_attribute("root", root))
}

pub fn upgrade_module(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    module_info: ModuleInfo,
    migrate_msg: Option<Binary>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;
    // Check if trying to upgrade this contract.
    if module_info.id() == MANAGER {
        return upgrade_self(deps, env, module_info, migrate_msg.unwrap());
    }
    let old_module_addr = OS_MODULES.load(deps.storage, &module_info.id())?;
    let contract = query_module_version(&deps.as_ref(), old_module_addr.clone())?;
    let module_ref = get_module(deps.as_ref(), module_info.clone(), Some(contract))?;
    match module_ref {
        ModuleReference::Extension(addr) => {
            // Update the address of the API internally
            update_module_addresses(
                deps.branch(),
                Some(vec![(module_info.id(), addr.to_string())]),
                None,
            )?;
            // replace it
            replace_api(deps, addr, old_module_addr)
        }
        ModuleReference::App(code_id) => {
            migrate_module(deps, env, old_module_addr, code_id, migrate_msg.unwrap())
        }
        ModuleReference::Service(code_id) => {
            migrate_module(deps, env, old_module_addr, code_id, migrate_msg.unwrap())
        }
        _ => Err(ManagerError::NotUpgradeable(module_info)),
    }
}
// migrates the module to a new version
fn migrate_module(
    _deps: DepsMut,
    _env: Env,
    module_addr: Addr,
    new_code_id: u64,
    migrate_msg: Binary,
) -> ManagerResult {
    let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: module_addr.into_string(),
        new_code_id,
        msg: migrate_msg,
    });
    Ok(Response::new().add_message(migration_msg))
}

/// Replaces the current API with a different version
/// Also moves all the trader permissions to the new contract and removes them from the old
pub fn replace_api(deps: DepsMut, new_api_addr: Addr, old_api_addr: Addr) -> ManagerResult {
    let mut msgs = vec![];
    // Makes sure we already have the API installed
    let proxy_addr = OS_MODULES.load(deps.storage, PROXY)?;
    let traders: TradersResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: old_api_addr.to_string(),
        msg: to_binary(&<ApiQuery<Empty>>::Base(BaseQueryMsg::Traders {
            proxy_address: proxy_addr.to_string(),
        }))?,
    }))?;
    let traders_to_migrate: Vec<String> = traders
        .traders
        .into_iter()
        .map(|addr| addr.into_string())
        .collect();
    // Remove traders from old
    msgs.push(configure_api(
        old_api_addr.to_string(),
        BaseExecuteMsg::UpdateTraders {
            to_add: None,
            to_remove: Some(traders_to_migrate.clone()),
        },
    )?);
    // Remove api as trader on dependencies
    msgs.push(configure_api(
        old_api_addr.to_string(),
        BaseExecuteMsg::Remove {},
    )?);
    // Add traders to new
    msgs.push(configure_api(
        new_api_addr.clone(),
        BaseExecuteMsg::UpdateTraders {
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
        new_api_addr.to_string(),
    )?);

    Ok(Response::new().add_messages(msgs))
}

/// Update the OS information
pub fn update_info(
    deps: DepsMut,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;
    let mut info: OsInfo = INFO.load(deps.storage)?;
    if let Some(name) = name {
        // validate address format
        info.name = name;
    }
    info.description = description;
    info.link = link;
    INFO.save(deps.storage, &info)?;
    Ok(Response::new())
}
pub fn update_os_status(deps: DepsMut, info: MessageInfo, new_status: Subscribed) -> ManagerResult {
    let config = CONFIG.load(deps.storage)?;

    if let Some(sub_addr) = config.subscription_address {
        if sub_addr.eq(&info.sender) {
            STATUS.save(deps.storage, &new_status)?;
            return Ok(Response::new().add_attribute("new_status", new_status.to_string()));
        }
    }
    Err(ManagerError::CallerNotSubscriptionContract {})
}

pub fn enable_ibc(deps: DepsMut, msg_info: MessageInfo, new_status: bool) -> ManagerResult {
    // Only root can update IBC status
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let maybe_client = OS_MODULES.may_load(deps.storage, IBC_CLIENT)?;
    let proxy = OS_MODULES.load(deps.storage, PROXY)?;
    let msg = if let Some(ibc_client) = maybe_client {
        // we have an IBC client so can't add more
        if new_status {
            return Err(ManagerError::ModuleAlreadyAdded {});
        }

        let remove_from_proxy_msg =
            remove_dapp_from_proxy(deps.as_ref(), proxy.into_string(), ibc_client.into_string())?;
        OS_MODULES.remove(deps.storage, IBC_CLIENT);
        remove_from_proxy_msg
    } else {
        if !new_status {
            return Err(ManagerError::Std(StdError::generic_err(
                "ibc_client is not installed",
            )));
        }
        let ibc_client = get_module(
            deps.as_ref(),
            ModuleInfo::from_id(IBC_CLIENT, ModuleVersion::Latest {})?,
            None,
        )?;
        let ibc_client_addr = match ibc_client {
            ModuleReference::Native(addr) => addr,
            _ => return Err(StdError::generic_err("ibc_client must be native contract").into()),
        };

        let add_to_proxy_msg = whitelist_dapp_on_proxy(
            deps.as_ref(),
            proxy.into_string(),
            ibc_client_addr.to_string(),
        )?;
        OS_MODULES.save(deps.storage, IBC_CLIENT, &ibc_client_addr)?;
        add_to_proxy_msg
    };

    Ok(Response::new()
        .add_message(msg)
        .add_attribute("action", "enable_ibc")
        .add_attribute("new_status", new_status.to_string()))
}

fn get_module(
    deps: Deps,
    module_info: ModuleInfo,
    old_contract: Option<ContractVersion>,
) -> Result<ModuleReference, ManagerError> {
    let config = CONFIG.load(deps.storage)?;
    match &module_info.version {
        ModuleVersion::Version(new_version) => {
            let old_contract = old_contract.unwrap();
            if new_version.parse::<Version>().unwrap()
                >= old_contract.version.parse::<Version>().unwrap()
            {
                Ok(MODULE_LIBRARY
                    .query(&deps.querier, config.version_control_address, module_info)?
                    .unwrap())
            } else {
                Err(ManagerError::OlderVersion(
                    new_version.to_owned(),
                    old_contract.version,
                ))
            }
        }
        ModuleVersion::Latest {} => {
            // Query latest version of contract
            let resp: ModuleResponse =
                deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                    contract_addr: config.version_control_address.to_string(),
                    msg: to_binary(&VersionQuery::Module {
                        module: module_info,
                    })?,
                }))?;
            Ok(resp.module.reference)
        }
    }
}

fn upgrade_self(
    deps: DepsMut,
    env: Env,
    module_info: ModuleInfo,
    migrate_msg: Binary,
) -> ManagerResult {
    let contract = get_contract_version(deps.storage)?;
    let mod_ref = get_module(deps.as_ref(), module_info.clone(), Some(contract))?;
    if let ModuleReference::App(manager_code_id) = mod_ref {
        let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: env.contract.address.into_string(),
            new_code_id: manager_code_id,
            msg: migrate_msg,
        });
        Ok(Response::new().add_message(migration_msg))
    } else {
        Err(ManagerError::InvalidReference(module_info))
    }
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
