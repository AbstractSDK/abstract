use abstract_macros::abstract_response;
use abstract_sdk::cw_helpers::AbstractAttributes;
use abstract_std::{
    account::{
        state::{
            AccountInfo, SuspensionStatus, ACCOUNT_ID, ACCOUNT_MODULES, CONFIG, DEPENDENTS, INFO,
            SUB_ACCOUNTS, SUSPENSION_STATUS,
        },
        ExecuteMsg, ModuleInstallConfig,
    },
    adapter::{
        AdapterBaseMsg, AuthorizedAddressesResponse, BaseExecuteMsg, BaseQueryMsg,
        ExecuteMsg as AdapterExecMsg, QueryMsg as AdapterQuery,
    },
    module_factory::{ExecuteMsg as ModuleFactoryMsg, FactoryModuleInstallConfig},
    objects::{
        dependency::Dependency,
        gov_type::GovernanceDetails,
        module::{assert_module_data_validity, Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        ownership::{self, GovOwnershipError},
        salt::generate_instantiate_salt,
        validation::{validate_description, validate_link, validate_name},
        version_control::VersionControlContract,
        AccountId,
    },
    version_control::ModuleResponse,
    ACCOUNT,
};
use cosmwasm_std::{
    ensure, from_json, to_json_binary, wasm_execute, Addr, Attribute, Binary, Coin, CosmosMsg,
    Deps, DepsMut, Empty, Env, MessageInfo, Response, StdError, StdResult, Storage, SubMsg,
    SubMsgResult, WasmMsg,
};
use cw2::{get_contract_version, ContractVersion};
use cw_storage_plus::Item;
use semver::Version;

use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
};

pub const REGISTER_MODULES_DEPENDENCIES: u64 = 1;
pub(crate) const INSTALL_MODULES_CONTEXT: Item<Vec<(Module, Option<Addr>)>> = Item::new("icontext");

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
    let mut add_to_proxy = Vec::with_capacity(modules.len());
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
                    add_to_proxy.push(module_address.to_string());
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
                    add_to_proxy.push(module_address.to_string());
                }
                add_to_manager.push((module.info.id(), module_address.to_string()));
                install_context.push((module.clone(), Some(module_address)));

                Some(init_msg.unwrap())
            }
            _ => return Err(AccountError::ModuleNotInstallable(module.info.to_string())),
        };
        manager_modules.push(FactoryModuleInstallConfig::new(module.info, init_msg_salt));
    }

    INSTALL_MODULES_CONTEXT.save(deps.storage, &install_context)?;

    let mut messages = vec![];

    // Add modules to proxy
    let proxy_addr = ACCOUNT_MODULES.load(deps.storage, ACCOUNT)?;
    if !add_to_proxy.is_empty() {
        messages.push(SubMsg::new(add_modules_to_proxy(
            proxy_addr.into_string(),
            add_to_proxy,
        )?));
    };

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
        REGISTER_MODULES_DEPENDENCIES,
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
            validation::validate_not_proxy(&id)?;
            ACCOUNT_MODULES.remove(deps.storage, id.as_str());
        }
    }

    Ok(AccountResponse::action("update_module_addresses"))
}

fn add_modules_to_proxy(
    proxy_address: String,
    module_addresses: Vec<String>,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(wasm_execute(
        proxy_address,
        &ProxyMsg::AddModules {
            modules: module_addresses,
        },
        vec![],
    )?
    .into())
}
