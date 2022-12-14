use abstract_sdk::feature_objects::VersionControlContract;
use abstract_sdk::os::{
    extension::{
        BaseExecuteMsg, BaseQueryMsg, ExecuteMsg as ExtensionExecMsg, QueryMsg as ExtensionQuery,
        TradersResponse,
    },
    manager::state::{OsInfo, Subscribed, CONFIG, INFO, OS_MODULES, ROOT, STATUS},
    module_factory::ExecuteMsg as ModuleFactoryMsg,
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
    },
    proxy::ExecuteMsg as TreasuryMsg,
    IBC_CLIENT,
};
use abstract_sdk::*;
use cosmwasm_std::{
    to_binary, wasm_execute, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty, Env, MessageInfo,
    QueryRequest, Response, StdError, StdResult, WasmMsg, WasmQuery,
};
use cw2::{get_contract_version, ContractVersion};
use cw_storage_plus::Item;
use os::manager::state::DEPENDENTS;
use os::manager::{CallbackMsg, ExecuteMsg};
use os::objects::dependency::Dependency;
use semver::Version;

use crate::versioning;
use crate::{
    contract::ManagerResult, error::ManagerError, queries::query_module_version,
    validators::validate_name_or_gov_type,
};
use abstract_sdk::os::{MANAGER, PROXY};

pub(crate) const MIGRATE_CONTEXT: Item<Vec<(String, Vec<Dependency>)>> = Item::new("context");

/// Adds, updates or removes provided addresses.
/// Should only be called by contract that adds/removes modules.
/// Factory is admin on init
pub fn update_module_addresses(
    deps: DepsMut,
    to_add: Option<Vec<(String, String)>>,
    to_remove: Option<Vec<String>>,
) -> ManagerResult {
    if let Some(modules_to_add) = to_add {
        for (id, new_address) in modules_to_add.into_iter() {
            if id.is_empty() {
                return Err(ManagerError::InvalidModuleName {});
            };
            // validate addr
            OS_MODULES.save(
                deps.storage,
                id.as_str(),
                &deps.api.addr_validate(&new_address)?,
            )?;
        }
    }

    if let Some(modules_to_remove) = to_remove {
        for id in modules_to_remove.into_iter() {
            OS_MODULES.remove(deps.storage, id.as_str());
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
    if OS_MODULES.may_load(deps.storage, &module.id())?.is_some() {
        return Err(ManagerError::ModuleAlreadyAdded {});
    }

    let config = CONFIG.load(deps.storage)?;

    let response = Response::new().add_message(CosmosMsg::Wasm(WasmMsg::Execute {
        contract_addr: config.module_factory_address.into(),
        msg: to_binary(&ModuleFactoryMsg::InstallModule { module, init_msg })?,
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
        Module {
            reference: ModuleReference::App(_),
            info,
        } => {
            let id = info.id();
            // assert version requirements
            let dependencies = versioning::assert_install_requirements(deps.as_ref(), &id)?;
            versioning::set_as_dependent(deps.storage, id, dependencies)?;
            response = response.add_message(whitelist_dapp_on_proxy(
                deps.as_ref(),
                proxy_addr.into_string(),
                module_address,
            )?)
        }
        Module {
            reference: ModuleReference::Extension(_),
            info,
        } => {
            let id = info.id();
            // assert version requirements
            let dependencies = versioning::assert_install_requirements(deps.as_ref(), &id)?;
            versioning::set_as_dependent(deps.storage, id, dependencies)?;
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
    module_id: String,
    exec_msg: Binary,
) -> ManagerResult {
    // Only root can update module configs
    ROOT.assert_admin(deps.as_ref(), &msg_info.sender)?;
    let module_addr = OS_MODULES.load(deps.storage, &module_id)?;

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
    // module can only be removed if there are no dependencies on it
    let dependents: Vec<String> = DEPENDENTS
        .load(deps.storage, &module_id)?
        .into_iter()
        .collect();
    if !dependents.is_empty() {
        return Err(ManagerError::ModuleHasDependents(dependents));
    }

    // Remove module as dependant from its dependencies.
    let module_dependencies = versioning::module_dependencies(deps.as_ref(), &module_id)?;
    versioning::remove_as_dependent(deps.storage, &module_id, module_dependencies)?;

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

/// Migrate modules through address updates or contract migrations
/// The dependency store is updated during migration
/// A reply message is called after performing all the migrations which ensures version compatibility of the new state.
/// Migrations are performed in-order and should be done in a top-down approach.
pub fn upgrade_modules(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    modules: Vec<(ModuleInfo, Option<Binary>)>,
) -> ManagerResult {
    ROOT.assert_admin(deps.as_ref(), &info.sender)?;
    let mut upgrade_msgs = vec![];
    for (module_info, migrate_msg) in modules {
        if module_info.id() == MANAGER {
            return upgrade_self(deps, env, module_info, migrate_msg.unwrap());
        }
        set_migrate_msgs_and_context(deps.branch(), module_info, migrate_msg, &mut upgrade_msgs)?;
    }
    let callback_msg = wasm_execute(
        env.contract.address,
        &ExecuteMsg::Callback(CallbackMsg {}),
        vec![],
    )?;
    Ok(Response::new()
        .add_messages(upgrade_msgs)
        .add_message(callback_msg))
}

pub fn set_migrate_msgs_and_context(
    mut deps: DepsMut,
    module_info: ModuleInfo,
    migrate_msg: Option<Binary>,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ManagerError> {
    let old_module_addr = OS_MODULES.load(deps.storage, &module_info.id())?;
    let contract = query_module_version(&deps.as_ref(), old_module_addr.clone())?;
    let module = get_module(deps.as_ref(), module_info.clone(), Some(contract))?;
    let id = module_info.id();

    match module.reference {
        // upgrading an extension is done by moving the traders to the new contract address and updating the permissions on the proxy.
        ModuleReference::Extension(addr) => {
            versioning::assert_migrate_requirements(
                deps.as_ref(),
                &id,
                module.info.version.to_string().parse().unwrap(),
            )?;
            let old_deps = versioning::module_dependencies(deps.as_ref(), &id)?;
            // Update the address of the extension internally
            update_module_addresses(
                deps.branch(),
                Some(vec![(id.clone(), addr.to_string())]),
                None,
            )?;

            // Add module upgrade to reply context
            let update_context = |mut upgraded_modules: Vec<(String,Vec<Dependency>)>| -> StdResult<Vec<(String,Vec<Dependency>)>> {
                upgraded_modules.push((id,old_deps));
                Ok(upgraded_modules)
            };
            MIGRATE_CONTEXT.update(deps.storage, update_context)?;

            msgs.append(replace_extension(deps, addr, old_module_addr)?.as_mut());
        }
        ModuleReference::App(code_id) => {
            versioning::assert_migrate_requirements(
                deps.as_ref(),
                &module.info.id(),
                module.info.version.to_string().parse().unwrap(),
            )?;
            let old_deps = versioning::module_dependencies(deps.as_ref(), &id)?;

            // Add module upgrade to reply context
            let update_context = |mut upgraded_modules: Vec<(String,Vec<Dependency>)>| -> StdResult<Vec<(String,Vec<Dependency>)>> {
                upgraded_modules.push((id,old_deps));
                Ok(upgraded_modules)
            };
            MIGRATE_CONTEXT.update(deps.storage, update_context)?;

            msgs.push(get_migrate_msg(
                old_module_addr,
                code_id,
                migrate_msg.unwrap_or_else(|| to_binary(&Empty {}).unwrap()),
            ));
        }
        ModuleReference::Standalone(code_id) => msgs.push(get_migrate_msg(
            old_module_addr,
            code_id,
            migrate_msg.unwrap(),
        )),
        _ => return Err(ManagerError::NotUpgradeable(module_info)),
    };
    Ok(())
}
// migrates the module to a new version
fn get_migrate_msg(module_addr: Addr, new_code_id: u64, migrate_msg: Binary) -> CosmosMsg {
    let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: module_addr.into_string(),
        new_code_id,
        msg: migrate_msg,
    });
    migration_msg
}

/// Replaces the current extension with a different version
/// Also moves all the trader permissions to the new contract and removes them from the old
pub fn replace_extension(
    deps: DepsMut,
    new_extension_addr: Addr,
    old_extension_addr: Addr,
) -> Result<Vec<CosmosMsg>, ManagerError> {
    let mut msgs = vec![];
    // Makes sure we already have the extension installed
    let proxy_addr = OS_MODULES.load(deps.storage, PROXY)?;
    let traders: TradersResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: old_extension_addr.to_string(),
        msg: to_binary(&<ExtensionQuery<Empty>>::Base(BaseQueryMsg::Traders {
            proxy_address: proxy_addr.to_string(),
        }))?,
    }))?;
    let traders_to_migrate: Vec<String> = traders
        .traders
        .into_iter()
        .map(|addr| addr.into_string())
        .collect();
    // Remove traders from old
    msgs.push(configure_extension(
        &old_extension_addr,
        BaseExecuteMsg::UpdateTraders {
            to_add: None,
            to_remove: Some(traders_to_migrate.clone()),
        },
    )?);
    // Remove extension as trader on dependencies
    msgs.push(configure_extension(
        &old_extension_addr,
        BaseExecuteMsg::Remove {},
    )?);
    // Add traders to new
    msgs.push(configure_extension(
        &new_extension_addr,
        BaseExecuteMsg::UpdateTraders {
            to_add: Some(traders_to_migrate),
            to_remove: None,
        },
    )?);
    // Remove extension permissions from proxy
    msgs.push(remove_dapp_from_proxy(
        deps.as_ref(),
        proxy_addr.to_string(),
        old_extension_addr.into_string(),
    )?);
    // Add new extension to proxy
    msgs.push(whitelist_dapp_on_proxy(
        deps.as_ref(),
        proxy_addr.into_string(),
        new_extension_addr.into_string(),
    )?);

    Ok(msgs)
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
        let ibc_client_addr = match ibc_client.reference {
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
) -> Result<Module, ManagerError> {
    let config = CONFIG.load(deps.storage)?;
    // Construct feature object to access registry functions
    let binding = VersionControlContract {
        contract_address: config.version_control_address,
    };
    let version_registry = binding.version_register(deps);
    match &module_info.version {
        ModuleVersion::Version(new_version) => {
            let old_contract = old_contract.unwrap();
            if new_version.parse::<Version>().unwrap()
                >= old_contract.version.parse::<Version>().unwrap()
            {
                Ok(Module {
                    info: module_info.clone(),
                    reference: version_registry.get_module_reference_raw(module_info)?,
                })
            } else {
                Err(ManagerError::OlderVersion(
                    new_version.to_owned(),
                    old_contract.version,
                ))
            }
        }
        ModuleVersion::Latest {} => {
            // Query latest version of contract
            version_registry.get_module(module_info).map_err(Into::into)
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
    let module = get_module(deps.as_ref(), module_info.clone(), Some(contract))?;
    if let ModuleReference::App(manager_code_id) = module.reference {
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
#[inline(always)]
fn configure_extension(
    extension_address: impl Into<String>,
    message: BaseExecuteMsg,
) -> StdResult<CosmosMsg> {
    let extension_msg: ExtensionExecMsg<Empty> = message.into();
    Ok(wasm_execute(extension_address, &extension_msg, vec![])?.into())
}
