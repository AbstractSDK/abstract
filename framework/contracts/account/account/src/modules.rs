use abstract_std::{
    account::{
        state::{ACCOUNT_ID, ACCOUNT_MODULES, CONFIG, DEPENDENTS, WHITELISTED_MODULES},
        ModuleInstallConfig,
    },
    adapter::{AdapterBaseMsg, BaseExecuteMsg, ExecuteMsg as AdapterExecMsg},
    module_factory::{ExecuteMsg as ModuleFactoryMsg, FactoryModuleInstallConfig},
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        ownership::{self},
        salt::generate_instantiate_salt,
        version_control::VersionControlContract,
    },
    version_control::ModuleResponse,
};
use cosmwasm_std::{
    ensure, wasm_execute, Addr, Attribute, Binary, Coin, CosmosMsg, Deps, DepsMut, MessageInfo,
    StdResult, Storage, SubMsg, WasmMsg,
};
use cw2::ContractVersion;
use cw_storage_plus::Item;
use semver::Version;

use crate::{
    contract::{AccountResponse, AccountResult, REGISTER_MODULES_DEPENDENCIES_REPLY_ID},
    error::AccountError,
};

pub use migration::MIGRATE_CONTEXT;
pub(crate) const INSTALL_MODULES_CONTEXT: Item<Vec<(Module, Option<Addr>)>> = Item::new("icontext");

pub mod migration;

const LIST_SIZE_LIMIT: usize = 15;

/// Attempts to install a new module through the Module Factory Contract
pub fn install_modules(
    mut deps: DepsMut,
    info: MessageInfo,
    modules: Vec<ModuleInstallConfig>,
) -> AccountResult {
    // only owner can call this method
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let config = CONFIG.load(deps.storage)?;

    let (install_msgs, install_attribute) = _install_modules(
        deps.branch(),
        modules,
        config.module_factory_address,
        config.version_control_address,
        info.funds, // We forward all the funds to the module_factory address for them to use in the install
    )?;
    let response = AccountResponse::new("install_modules", std::iter::once(install_attribute))
        .add_submessages(install_msgs);

    Ok(response)
}

/// Generate message and attribute for installing module
/// Adds the modules to the internal store for reference and adds them to the proxy allowlist if applicable.
pub fn _install_modules(
    mut deps: DepsMut,
    modules: Vec<ModuleInstallConfig>,
    module_factory_address: Addr,
    version_control_address: Addr,
    funds: Vec<Coin>,
) -> AccountResult<(Vec<SubMsg>, Attribute)> {
    let mut installed_modules = Vec::with_capacity(modules.len());
    let mut manager_modules = Vec::with_capacity(modules.len());
    let account_id = ACCOUNT_ID.load(deps.storage)?;
    let version_control = VersionControlContract::new(version_control_address);

    let canonical_module_factory = deps
        .api
        .addr_canonicalize(module_factory_address.as_str())?;

    let (infos, init_msgs): (Vec<_>, Vec<_>) =
        modules.into_iter().map(|m| (m.module, m.init_msg)).unzip();
    let modules = version_control
        .query_modules_configs(infos, &deps.querier)
        .map_err(|error| AccountError::QueryModulesFailed { error })?;

    let mut install_context = Vec::with_capacity(modules.len());
    let mut add_to_whitelist = Vec::with_capacity(modules.len());
    let mut add_to_manager = Vec::with_capacity(modules.len());

    let salt: Binary = generate_instantiate_salt(&account_id);
    for (ModuleResponse { module, .. }, init_msg) in modules.into_iter().zip(init_msgs) {
        // Check if module is already enabled.
        if ACCOUNT_MODULES
            .may_load(deps.storage, &module.info.id())?
            .is_some()
        {
            return Err(AccountError::ModuleAlreadyInstalled(module.info.id()));
        }
        installed_modules.push(module.info.id_with_version());

        let init_msg_salt = match &module.reference {
            ModuleReference::Adapter(module_address)
            | ModuleReference::Native(module_address)
            | ModuleReference::Service(module_address) => {
                if module.should_be_whitelisted() {
                    add_to_whitelist.push(module_address.to_string());
                }
                add_to_manager.push((module.info.id(), module_address.to_string()));
                install_context.push((module.clone(), None));
                None
            }
            ModuleReference::App(code_id) | ModuleReference::Standalone(code_id) => {
                let checksum = deps.querier.query_wasm_code_info(*code_id)?.checksum;
                let module_address = cosmwasm_std::instantiate2_address(
                    checksum.as_slice(),
                    &canonical_module_factory,
                    &salt,
                )?;
                let module_address = deps.api.addr_humanize(&module_address)?;
                ensure!(
                    deps.querier
                        .query_wasm_contract_info(module_address.to_string())
                        .is_err(),
                    AccountError::ProhibitedReinstall {}
                );
                if module.should_be_whitelisted() {
                    add_to_whitelist.push(module_address.to_string());
                }
                add_to_manager.push((module.info.id(), module_address.to_string()));
                install_context.push((module.clone(), Some(module_address)));

                Some(init_msg.unwrap())
            }
            _ => return Err(AccountError::ModuleNotInstallable(module.info.to_string())),
        };
        manager_modules.push(FactoryModuleInstallConfig::new(module.info, init_msg_salt));
    }
    _whitelist_modules(deps.branch(), add_to_whitelist)?;

    INSTALL_MODULES_CONTEXT.save(deps.storage, &install_context)?;

    let mut messages = vec![];

    // Update module addrs
    update_module_addresses(deps.branch(), Some(add_to_manager), None)?;

    // Install modules message
    messages.push(SubMsg::reply_on_success(
        wasm_execute(
            module_factory_address,
            &ModuleFactoryMsg::InstallModules {
                modules: manager_modules,
                salt,
            },
            funds,
        )?,
        REGISTER_MODULES_DEPENDENCIES_REPLY_ID,
    ));

    Ok((
        messages,
        Attribute::new("installed_modules", format!("{installed_modules:?}")),
    ))
}

/// Adds, updates or removes provided addresses.
/// Should only be called by contract that adds/removes modules.
/// Factory is admin on init
pub fn update_module_addresses(
    deps: DepsMut,
    to_add: Option<Vec<(String, String)>>,
    to_remove: Option<Vec<String>>,
) -> AccountResult {
    if let Some(modules_to_add) = to_add {
        for (id, new_address) in modules_to_add.into_iter() {
            if id.is_empty() {
                return Err(AccountError::InvalidModuleName {});
            };
            // validate addr
            ACCOUNT_MODULES.save(
                deps.storage,
                id.as_str(),
                &deps.api.addr_validate(&new_address)?,
            )?;
        }
    }

    if let Some(modules_to_remove) = to_remove {
        for id in modules_to_remove.into_iter() {
            ACCOUNT_MODULES.remove(deps.storage, id.as_str());
        }
    }

    Ok(AccountResponse::action("update_module_addresses"))
}

/// Uninstall the module with the ID [`module_id`]
pub fn uninstall_module(mut deps: DepsMut, info: MessageInfo, module_id: String) -> AccountResult {
    // only owner can uninstall modules
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    // module can only be uninstalled if there are no dependencies on it
    let dependents = DEPENDENTS.may_load(deps.storage, &module_id)?;
    if let Some(dependents) = dependents {
        if !dependents.is_empty() {
            return Err(AccountError::ModuleHasDependents(Vec::from_iter(
                dependents,
            )));
        }
        // Remove the module from the dependents list
        DEPENDENTS.remove(deps.storage, &module_id);
    }

    // Remove module as dependant from its dependencies.
    let module_data = crate::versioning::load_module_data(deps.as_ref(), &module_id)?;
    let module_dependencies = module_data.dependencies;
    crate::versioning::remove_as_dependent(deps.storage, &module_id, module_dependencies)?;

    // Remove for proxy if needed
    let config = CONFIG.load(deps.storage)?;
    let vc = VersionControlContract::new(config.version_control_address);

    let module = vc.query_module(
        ModuleInfo::from_id(&module_data.module, module_data.version.into())?,
        &deps.querier,
    )?;

    // Remove module from whitelist if it supposed to be removed
    if module.should_be_whitelisted() {
        _remove_whitelist_module(deps.branch(), module_id.clone())?;
    }
    ACCOUNT_MODULES.remove(deps.storage, &module_id);

    let response = AccountResponse::new("uninstall_module", vec![("module", &module_id)]);
    Ok(response)
}

/// Execute the [`exec_msg`] on the provided [`module_id`],
pub fn exec_on_module(
    deps: DepsMut,
    info: MessageInfo,
    module_id: String,
    exec_msg: Binary,
) -> AccountResult {
    // only owner can forward messages to modules
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let module_addr = load_module_addr(deps.storage, &module_id)?;

    let response = AccountResponse::new("exec_on_module", vec![("module", module_id)]).add_message(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: module_addr.into(),
            msg: exec_msg,
            funds: info.funds,
        }),
    );

    Ok(response)
}

/// Checked load of a module address
pub fn load_module_addr(storage: &dyn Storage, module_id: &String) -> AccountResult<Addr> {
    ACCOUNT_MODULES
        .may_load(storage, module_id)?
        .ok_or_else(|| AccountError::ModuleNotFound(module_id.clone()))
}

/// Query Version Control for the [`Module`] given the provided [`ContractVersion`]
pub fn query_module(
    deps: Deps,
    module_info: ModuleInfo,
    old_contract_version: Option<ContractVersion>,
) -> Result<ModuleResponse, AccountError> {
    let config = CONFIG.load(deps.storage)?;
    // Construct feature object to access registry functions
    let version_control = VersionControlContract::new(config.version_control_address);

    let module = match &module_info.version {
        ModuleVersion::Version(new_version) => {
            let old_contract = old_contract_version.unwrap();

            let new_version = new_version.parse::<Version>().unwrap();
            let old_version = old_contract.version.parse::<Version>().unwrap();

            if new_version < old_version {
                return Err(AccountError::OlderVersion(
                    new_version.to_string(),
                    old_version.to_string(),
                ));
            }
            Module {
                info: module_info.clone(),
                reference: version_control
                    .query_module_reference_raw(&module_info, &deps.querier)?,
            }
        }
        ModuleVersion::Latest => {
            // Query latest version of contract
            version_control.query_module(module_info.clone(), &deps.querier)?
        }
    };

    Ok(ModuleResponse {
        module: Module {
            info: module.info,
            reference: module.reference,
        },
        config: version_control.query_config(module_info, &deps.querier)?,
    })
}

#[inline(always)]
fn configure_adapter(
    adapter_address: impl Into<String>,
    message: AdapterBaseMsg,
) -> StdResult<CosmosMsg> {
    let adapter_msg: AdapterExecMsg = BaseExecuteMsg {
        proxy_address: None,
        msg: message,
    }
    .into();
    Ok(wasm_execute(adapter_address, &adapter_msg, vec![])?.into())
}

/// Add a contract to the whitelist
fn _whitelist_modules(deps: DepsMut, modules: Vec<String>) -> AccountResult<()> {
    let mut whitelisted_modules = WHITELISTED_MODULES.load(deps.storage)?;

    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if whitelisted_modules.0.len() >= LIST_SIZE_LIMIT {
        return Err(AccountError::ModuleLimitReached {});
    }

    for module in modules.iter() {
        let module_addr = deps.api.addr_validate(module)?;

        if whitelisted_modules.0.contains(&module_addr) {
            return Err(AccountError::AlreadyWhitelisted(module.clone()));
        }

        // Add contract to whitelist.
        whitelisted_modules.0.push(module_addr);
    }

    WHITELISTED_MODULES.save(deps.storage, &whitelisted_modules)?;

    Ok(())
}

/// Remove a contract from the whitelist
fn _remove_whitelist_module(deps: DepsMut, module: String) -> AccountResult<()> {
    WHITELISTED_MODULES.update(deps.storage, |mut whitelisted_modules| {
        let module_address = deps.api.addr_validate(&module)?;

        if !whitelisted_modules.0.contains(&module_address) {
            return Err(AccountError::NotWhitelisted(module.clone()));
        }
        // Remove contract from whitelist.
        whitelisted_modules.0.retain(|addr| *addr != module_address);
        Ok(whitelisted_modules)
    })?;

    Ok(())
}
