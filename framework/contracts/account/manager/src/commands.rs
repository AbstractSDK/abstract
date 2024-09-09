use abstract_macros::abstract_response;
use abstract_sdk::cw_helpers::AbstractAttributes;
use abstract_std::{
    account::{state::ACCOUNT_ID, ExecuteMsg as ProxyMsg},
    account::{
        state::{
            AccountInfo, SuspensionStatus, ACCOUNT_MODULES, CONFIG, DEPENDENTS, INFO, SUB_ACCOUNTS,
            SUSPENSION_STATUS,
        },
        CallbackMsg, ExecuteMsg, InternalConfigAction, ModuleInstallConfig, UpdateSubAccountAction,
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
    contract::ManagerResult, error::ManagerError, queries::query_module_version, validation,
    versioning,
};

pub const REGISTER_MODULES_DEPENDENCIES: u64 = 1;

pub const MIGRATE_CONTEXT: Item<Vec<(String, Vec<Dependency>)>> = Item::new("context");

pub(crate) const INSTALL_MODULES_CONTEXT: Item<Vec<(Module, Option<Addr>)>> = Item::new("icontext");

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

    Ok(ManagerResponse::action("update_module_addresses"))
}

/// Adds the modules dependencies
pub(crate) fn register_dependencies(deps: DepsMut, _result: SubMsgResult) -> ManagerResult {
    let modules = INSTALL_MODULES_CONTEXT.load(deps.storage)?;

    for (module, module_addr) in &modules {
        assert_module_data_validity(&deps.querier, module, module_addr.clone())?;

        match module {
            Module {
                reference: ModuleReference::App(_),
                info,
            }
            | Module {
                reference: ModuleReference::Adapter(_),
                info,
            } => {
                let id = info.id();
                // assert version requirements
                let dependencies = versioning::assert_install_requirements(deps.as_ref(), &id)?;
                versioning::set_as_dependent(deps.storage, id, dependencies)?;
            }
            Module {
                reference: ModuleReference::Standalone(_),
                info,
            } => {
                let id = info.id();
                // assert version requirements
                let dependencies =
                    versioning::assert_install_requirements_standalone(deps.as_ref(), &id)?;
                versioning::set_as_dependent(deps.storage, id, dependencies)?;
            }
            _ => (),
        };
    }

    Ok(Response::new())
}

/// Execute the [`exec_msg`] on the provided [`module_id`],
pub fn exec_on_module(
    deps: DepsMut,
    info: MessageInfo,
    module_id: String,
    exec_msg: Binary,
) -> ManagerResult {
    // only owner can forward messages to modules
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let module_addr = load_module_addr(deps.storage, &module_id)?;

    let response = ManagerResponse::new("exec_on_module", vec![("module", module_id)]).add_message(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: module_addr.into(),
            msg: exec_msg,
            funds: info.funds,
        }),
    );

    Ok(response)
}

#[allow(clippy::too_many_arguments)]
/// Creates a sub-account for this account,
pub fn create_sub_account(
    deps: DepsMut,
    info: MessageInfo,
    env: Env,
    name: String,
    description: Option<String>,
    link: Option<String>,
    namespace: Option<String>,
    install_modules: Vec<ModuleInstallConfig>,
    account_id: Option<u32>,
) -> ManagerResult {
    // only owner can create a subaccount
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let create_account_msg = &abstract_std::account_factory::ExecuteMsg::CreateAccount {
        // proxy of this manager will be the account owner
        governance: GovernanceDetails::SubAccount {
            account: env.contract.address.into_string(),
        },
        name,
        description,
        link,
        namespace,
        install_modules,
        account_id: account_id.map(AccountId::local),
    };

    let account_factory_addr = query_module(
        deps.as_ref(),
        ModuleInfo::from_id_latest(abstract_std::ACCOUNT_FACTORY)?,
        None,
    )?
    .module
    .reference
    .unwrap_native()?;

    // Call factory and attach all funds that were provided.
    let account_creation_message =
        wasm_execute(account_factory_addr, create_account_msg, info.funds)?;

    let response = ManagerResponse::new::<_, Attribute>("create_sub_account", vec![])
        .add_message(account_creation_message);

    Ok(response)
}

pub fn handle_sub_account_action(
    deps: DepsMut,
    info: MessageInfo,
    action: UpdateSubAccountAction,
) -> ManagerResult {
    match action {
        UpdateSubAccountAction::UnregisterSubAccount { id } => {
            unregister_sub_account(deps, info, id)
        }
        UpdateSubAccountAction::RegisterSubAccount { id } => register_sub_account(deps, info, id),
        _ => unimplemented!(),
    }
}

// Unregister sub-account from the state
fn unregister_sub_account(deps: DepsMut, info: MessageInfo, id: u32) -> ManagerResult {
    let config = CONFIG.load(deps.storage)?;

    let account = abstract_std::version_control::state::ACCOUNT_ADDRESSES.query(
        &deps.querier,
        config.version_control_address,
        &AccountId::local(id),
    )?;

    if account.is_some_and(|a| a.addr() == info.sender) {
        SUB_ACCOUNTS.remove(deps.storage, id);

        Ok(ManagerResponse::new(
            "unregister_sub_account",
            vec![("sub_account_removed", id.to_string())],
        ))
    } else {
        Err(ManagerError::SubAccountRemovalFailed {})
    }
}

// Register sub-account to the state
fn register_sub_account(deps: DepsMut, info: MessageInfo, id: u32) -> ManagerResult {
    let config = CONFIG.load(deps.storage)?;

    let account = abstract_std::version_control::state::ACCOUNT_ADDRESSES.query(
        &deps.querier,
        config.version_control_address,
        &AccountId::local(id),
    )?;

    if account.is_some_and(|a| a.addr() == info.sender) {
        SUB_ACCOUNTS.save(deps.storage, id, &Empty {})?;

        Ok(ManagerResponse::new(
            "register_sub_account",
            vec![("sub_account_added", id.to_string())],
        ))
    } else {
        Err(ManagerError::SubAccountRegisterFailed {})
    }
}

/// Checked load of a module address
fn load_module_addr(storage: &dyn Storage, module_id: &String) -> Result<Addr, ManagerError> {
    ACCOUNT_MODULES
        .may_load(storage, module_id)?
        .ok_or_else(|| ManagerError::ModuleNotFound(module_id.clone()))
}

/// Uninstall the module with the ID [`module_id`]
pub fn uninstall_module(deps: DepsMut, info: MessageInfo, module_id: String) -> ManagerResult {
    // only owner can uninstall modules
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    validation::validate_not_proxy(&module_id)?;

    // module can only be uninstalled if there are no dependencies on it
    let dependents = DEPENDENTS.may_load(deps.storage, &module_id)?;
    if let Some(dependents) = dependents {
        if !dependents.is_empty() {
            return Err(ManagerError::ModuleHasDependents(Vec::from_iter(
                dependents,
            )));
        }
        // Remove the module from the dependents list
        DEPENDENTS.remove(deps.storage, &module_id);
    }

    // Remove module as dependant from its dependencies.
    let module_data = versioning::load_module_data(deps.as_ref(), &module_id)?;
    let module_dependencies = module_data.dependencies;
    versioning::remove_as_dependent(deps.storage, &module_id, module_dependencies)?;

    // Remove for proxy if needed
    let config = CONFIG.load(deps.storage)?;
    let vc = VersionControlContract::new(config.version_control_address);

    let module = vc.query_module(
        ModuleInfo::from_id(&module_data.module, module_data.version.into())?,
        &deps.querier,
    )?;

    let mut response = ManagerResponse::new("uninstall_module", vec![("module", &module_id)]);
    // Remove module from proxy whitelist if it supposed to be removed
    if module.should_be_whitelisted() {
        let proxy = ACCOUNT_MODULES.load(deps.storage, ACCOUNT)?;
        let module_addr = load_module_addr(deps.storage, &module_id)?;
        let remove_from_proxy_msg =
            remove_module_from_proxy(proxy.into_string(), module_addr.into_string())?;
        response = response.add_message(remove_from_proxy_msg);
    }
    ACCOUNT_MODULES.remove(deps.storage, &module_id);

    Ok(response)
}

/// Update governance of sub_accounts account after claim
pub fn maybe_update_sub_account_governance(deps: DepsMut) -> ManagerResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];
    let mut account_id = None;
    let ownership = ownership::get_ownership(deps.storage)?;
    // Get pending governance
    let pending_governance = ownership
        .pending_owner
        .ok_or(GovOwnershipError::TransferNotFound)?;

    // Clear state for previous manager if it was sub-account
    if let GovernanceDetails::SubAccount { account } = ownership.owner {
        let id = ACCOUNT_ID.load(deps.storage)?;
        let unregister_message = wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::UnregisterSubAccount {
                id: id.seq(),
            }),
            vec![],
        )?;
        // For optimizing the gas we save it, in case new owner is sub-account as well
        account_id = Some(id);
        msgs.push(unregister_message.into());
    }

    // Update state for new manager if owner will be the sub-account
    if let GovernanceDetails::SubAccount { account } = &pending_governance {
        let id = if let Some(id) = account_id {
            id
        } else {
            ACCOUNT_ID.load(deps.storage)?
        };
        let register_message = wasm_execute(
            account,
            &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::RegisterSubAccount {
                id: id.seq(),
            }),
            vec![],
        )?;
        msgs.push(register_message.into());
    }

    Ok(msgs)
}

/// Renounce ownership of this account \
/// **WARNING**: This will lock the account, making it unusable.
pub fn remove_account_from_contracts(deps: DepsMut) -> ManagerResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    let account_id = ACCOUNT_ID.load(deps.storage)?;
    // Check for any sub accounts
    let sub_account = SUB_ACCOUNTS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .next()
        .transpose()?;
    ensure!(
        sub_account.is_none(),
        ManagerError::RenounceWithSubAccount {}
    );

    let ownership = ownership::get_ownership(deps.storage)?;
    if let GovernanceDetails::SubAccount { account } = ownership.owner {
        // Unregister itself (sub-account) from the owning account.
        msgs.push(
            wasm_execute(
                account,
                &ExecuteMsg::UpdateSubAccount(UpdateSubAccountAction::UnregisterSubAccount {
                    id: account_id.seq(),
                }),
                vec![],
            )?
            .into(),
        );
    }

    let config = CONFIG.load(deps.storage)?;
    let vc = VersionControlContract::new(config.version_control_address);
    let mut namespaces = vc
        .query_namespaces(vec![account_id], &deps.querier)?
        .namespaces;
    let namespace = namespaces.pop();
    if let Some((namespace, _)) = namespace {
        // Remove the namespace that this account holds.
        msgs.push(
            wasm_execute(
                vc.address,
                &abstract_std::version_control::ExecuteMsg::RemoveNamespaces {
                    namespaces: vec![namespace.to_string()],
                },
                vec![],
            )?
            .into(),
        )
    };
    Ok(msgs)
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
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;
    ensure!(!modules.is_empty(), ManagerError::NoUpdates {});

    let mut upgrade_msgs = vec![];

    let mut manager_migrate_info = None;

    let mut upgraded_module_ids = Vec::new();

    // Set the migrate messages for each module that's not the manager and update the dependency store
    for (module_info, migrate_msg) in modules {
        let module_id = module_info.id();

        // Check for duplicates
        if upgraded_module_ids.contains(&module_id) {
            return Err(ManagerError::DuplicateModuleMigration { module_id });
        } else {
            upgraded_module_ids.push(module_id.clone());
        }

        if module_id == ACCOUNT {
            manager_migrate_info = Some((module_info, migrate_msg));
        } else {
            set_migrate_msgs_and_context(
                deps.branch(),
                module_info,
                migrate_msg,
                &mut upgrade_msgs,
            )?;
        }
    }

    // Upgrade the manager last
    if let Some((manager_info, manager_migrate_msg)) = manager_migrate_info {
        upgrade_msgs.push(self_upgrade_msg(
            deps.branch(),
            &env.contract.address,
            manager_info,
            manager_migrate_msg.unwrap_or_default(),
        )?);
    }

    let callback_msg = wasm_execute(
        env.contract.address,
        &ExecuteMsg::Callback(CallbackMsg {}),
        vec![],
    )?;

    Ok(ManagerResponse::new(
        "upgrade_modules",
        vec![("upgraded_modules", upgraded_module_ids.join(","))],
    )
    .add_messages(upgrade_msgs)
    .add_message(callback_msg))
}

pub(crate) fn set_migrate_msgs_and_context(
    deps: DepsMut,
    module_info: ModuleInfo,
    migrate_msg: Option<Binary>,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ManagerError> {
    let config = CONFIG.load(deps.storage)?;
    let version_control = VersionControlContract::new(config.version_control_address);

    let old_module_addr = load_module_addr(deps.storage, &module_info.id())?;
    let old_module_cw2 =
        query_module_version(deps.as_ref(), old_module_addr.clone(), &version_control)?;
    let requested_module = query_module(deps.as_ref(), module_info.clone(), Some(old_module_cw2))?;

    let migrate_msgs = match requested_module.module.reference {
        // upgrading an adapter is done by moving the authorized addresses to the new contract address and updating the permissions on the proxy.
        ModuleReference::Adapter(new_adapter_addr) => handle_adapter_migration(
            deps,
            requested_module.module.info,
            old_module_addr,
            new_adapter_addr,
        )?,
        ModuleReference::App(code_id) => handle_app_migration(
            deps,
            migrate_msg,
            old_module_addr,
            requested_module.module.info,
            code_id,
        )?,
        ModuleReference::AccountBase(code_id) | ModuleReference::Standalone(code_id) => {
            vec![build_module_migrate_msg(
                old_module_addr,
                code_id,
                migrate_msg.unwrap(),
            )]
        }

        _ => return Err(ManagerError::NotUpgradeable(module_info)),
    };
    msgs.extend(migrate_msgs);
    Ok(())
}

/// Handle Adapter module migration and return the migration messages
fn handle_adapter_migration(
    mut deps: DepsMut,
    module_info: ModuleInfo,
    old_adapter_addr: Addr,
    new_adapter_addr: Addr,
) -> ManagerResult<Vec<CosmosMsg>> {
    let module_id = module_info.id();
    versioning::assert_migrate_requirements(
        deps.as_ref(),
        &module_id,
        module_info.version.try_into()?,
    )?;
    let old_deps = versioning::load_module_dependencies(deps.as_ref(), &module_id)?;
    // Update the address of the adapter internally
    update_module_addresses(
        deps.branch(),
        Some(vec![(module_id.clone(), new_adapter_addr.to_string())]),
        None,
    )?;

    add_module_upgrade_to_context(deps.storage, &module_id, old_deps)?;

    replace_adapter(deps, new_adapter_addr, old_adapter_addr)
}

/// Handle app module migration and return the migration messages
fn handle_app_migration(
    deps: DepsMut,
    migrate_msg: Option<Binary>,
    old_module_addr: Addr,
    module_info: ModuleInfo,
    code_id: u64,
) -> ManagerResult<Vec<CosmosMsg>> {
    let module_id = module_info.id();
    versioning::assert_migrate_requirements(
        deps.as_ref(),
        &module_id,
        module_info.version.try_into()?,
    )?;
    let old_deps = versioning::load_module_dependencies(deps.as_ref(), &module_id)?;

    // Add module upgrade to reply context
    add_module_upgrade_to_context(deps.storage, &module_id, old_deps)?;

    Ok(vec![build_module_migrate_msg(
        old_module_addr,
        code_id,
        migrate_msg.unwrap_or_else(|| to_json_binary(&Empty {}).unwrap()),
    )])
}

/// Add the module upgrade to the migration context and check for duplicates
fn add_module_upgrade_to_context(
    storage: &mut dyn Storage,
    module_id: &str,
    module_deps: Vec<Dependency>,
) -> Result<(), ManagerError> {
    // Add module upgrade to reply context
    let update_context = |mut upgraded_modules: Vec<(String, Vec<Dependency>)>| -> StdResult<Vec<(String, Vec<Dependency>)>> {
        upgraded_modules.push((module_id.to_string(), module_deps));
        Ok(upgraded_modules)
    };
    MIGRATE_CONTEXT.update(storage, update_context)?;

    Ok(())
}

// migrates the module to a new version
fn build_module_migrate_msg(module_addr: Addr, new_code_id: u64, migrate_msg: Binary) -> CosmosMsg {
    let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
        contract_addr: module_addr.into_string(),
        new_code_id,
        msg: migrate_msg,
    });
    migration_msg
}

/// Replaces the current adapter with a different version
/// Also moves all the authorized address permissions to the new contract and removes them from the old
pub fn replace_adapter(
    deps: DepsMut,
    new_adapter_addr: Addr,
    old_adapter_addr: Addr,
) -> Result<Vec<CosmosMsg>, ManagerError> {
    let mut msgs = vec![];
    // Makes sure we already have the adapter installed
    let proxy_addr = ACCOUNT_MODULES.load(deps.storage, ACCOUNT)?;
    let AuthorizedAddressesResponse {
        addresses: authorized_addresses,
    } = deps.querier.query_wasm_smart(
        old_adapter_addr.to_string(),
        &<AdapterQuery<Empty>>::Base(BaseQueryMsg::AuthorizedAddresses {
            proxy_address: proxy_addr.to_string(),
        }),
    )?;
    let authorized_to_migrate: Vec<String> = authorized_addresses
        .into_iter()
        .map(|addr| addr.into_string())
        .collect();
    // Remove authorized addresses
    msgs.push(configure_adapter(
        &old_adapter_addr,
        AdapterBaseMsg::UpdateAuthorizedAddresses {
            to_add: vec![],
            to_remove: authorized_to_migrate.clone(),
        },
    )?);
    // Add authorized addresses to new
    msgs.push(configure_adapter(
        &new_adapter_addr,
        AdapterBaseMsg::UpdateAuthorizedAddresses {
            to_add: authorized_to_migrate,
            to_remove: vec![],
        },
    )?);
    // Remove adapter permissions from proxy
    msgs.push(remove_module_from_proxy(
        proxy_addr.to_string(),
        old_adapter_addr.into_string(),
    )?);
    // Add new adapter to proxy
    msgs.push(add_modules_to_proxy(
        proxy_addr.into_string(),
        vec![new_adapter_addr.into_string()],
    )?);

    Ok(msgs)
}

/// Update the Account information
pub fn update_info(
    deps: DepsMut,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
) -> ManagerResult {
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let mut info: AccountInfo = INFO.load(deps.storage)?;
    if let Some(name) = name {
        validate_name(&name)?;
        info.name = name;
    }
    validate_description(description.as_deref())?;
    info.description = description;
    validate_link(link.as_deref())?;
    info.link = link;
    INFO.save(deps.storage, &info)?;

    Ok(ManagerResponse::action("update_info"))
}

pub fn update_suspension_status(
    deps: DepsMut,
    info: MessageInfo,
    is_suspended: SuspensionStatus,
    response: Response,
) -> ManagerResult {
    // only owner can update suspension status
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    SUSPENSION_STATUS.save(deps.storage, &is_suspended)?;

    Ok(response.add_abstract_attributes(vec![("is_suspended", is_suspended.to_string())]))
}

/// Query Version Control for the [`Module`] given the provided [`ContractVersion`]
fn query_module(
    deps: Deps,
    module_info: ModuleInfo,
    old_contract_version: Option<ContractVersion>,
) -> Result<ModuleResponse, ManagerError> {
    let config = CONFIG.load(deps.storage)?;
    // Construct feature object to access registry functions
    let version_control = VersionControlContract::new(config.version_control_address);

    let module = match &module_info.version {
        ModuleVersion::Version(new_version) => {
            let old_contract = old_contract_version.unwrap();

            let new_version = new_version.parse::<Version>().unwrap();
            let old_version = old_contract.version.parse::<Version>().unwrap();

            if new_version < old_version {
                return Err(ManagerError::OlderVersion(
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

fn self_upgrade_msg(
    deps: DepsMut,
    self_addr: &Addr,
    module_info: ModuleInfo,
    migrate_msg: Binary,
) -> ManagerResult<CosmosMsg> {
    let contract = get_contract_version(deps.storage)?;
    let module = query_module(deps.as_ref(), module_info.clone(), Some(contract))?;
    if let ModuleReference::AccountBase(manager_code_id) = module.module.reference {
        let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: self_addr.to_string(),
            new_code_id: manager_code_id,
            msg: migrate_msg,
        });
        Ok(migration_msg)
    } else {
        Err(ManagerError::InvalidReference(module_info))
    }
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

fn remove_module_from_proxy(
    proxy_address: String,
    dapp_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(wasm_execute(
        proxy_address,
        &ProxyMsg::RemoveModule {
            module: dapp_address,
        },
        vec![],
    )?
    .into())
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

pub fn update_account_status(
    deps: DepsMut,
    info: MessageInfo,
    suspension_status: Option<bool>,
) -> Result<Response, ManagerError> {
    let mut response = ManagerResponse::action("update_status");

    if let Some(suspension_status) = suspension_status {
        response = update_suspension_status(deps, info, suspension_status, response)?;
    } else {
        return Err(ManagerError::NoUpdates {});
    }

    Ok(response)
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::{contract, test_common::mock_init};
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage},
        Order, OwnedDeps, StdError,
    };
    use speculoos::prelude::*;

    type ManagerTestResult = Result<(), ManagerError>;

    fn mock_installed_proxy(deps: &mut MockDeps) -> StdResult<()> {
        let base = test_account_base(deps.api);
        ACCOUNT_MODULES.save(deps.as_mut().storage, ACCOUNT, &base.proxy)
    }

    fn execute_as(deps: DepsMut, sender: &Addr, msg: ExecuteMsg) -> ManagerResult {
        contract::execute(deps, mock_env(), message_info(sender, &[]), msg)
    }

    fn init_with_proxy(deps: &mut MockDeps) {
        mock_init(deps).unwrap();
        mock_installed_proxy(deps).unwrap();
    }

    fn load_account_modules(storage: &dyn Storage) -> Result<Vec<(String, Addr)>, StdError> {
        ACCOUNT_MODULES
            .range(storage, None, None, Order::Ascending)
            .collect()
    }

    fn test_only_owner(msg: ExecuteMsg) -> ManagerTestResult {
        let mut deps = mock_dependencies();
        let not_owner = deps.api.addr_make("not_owner");
        mock_init(&mut deps)?;

        let res = execute_as(deps.as_mut(), &not_owner, msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(ManagerError::Ownership(
                ownership::GovOwnershipError::NotOwner,
            ));

        Ok(())
    }

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    mod set_owner_and_gov_type {
        use ownership::GovAction;

        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let deps = mock_dependencies();
            let test_owner = deps.api.addr_make("test_owner");

            let msg = ExecuteMsg::UpdateOwnership(GovAction::TransferOwnership {
                new_owner: GovernanceDetails::Monarchy {
                    monarch: test_owner.to_string(),
                },
                expiry: None,
            });

            test_only_owner(msg)
        }

        #[test]
        fn validates_new_owner_address() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateOwnership(GovAction::TransferOwnership {
                new_owner: GovernanceDetails::Monarchy {
                    monarch: "INVALID".to_string(),
                },
                expiry: None,
            });

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(res).is_err().matches(|err| {
                matches!(
                    err,
                    ManagerError::Ownership(GovOwnershipError::Abstract(
                        abstract_std::AbstractError::Std(StdError::GenericErr { .. })
                    ))
                )
            });
            Ok(())
        }

        #[test]
        fn updates_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            let new_owner = deps.api.addr_make("new_owner");
            mock_init(&mut deps)?;

            let set_owner_msg = ExecuteMsg::UpdateOwnership(GovAction::TransferOwnership {
                new_owner: GovernanceDetails::Monarchy {
                    monarch: new_owner.to_string(),
                },
                expiry: None,
            });

            let res = execute_as(deps.as_mut(), &owner, set_owner_msg);
            assert_that!(&res).is_ok();

            let accept_msg = ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership);
            execute_as(deps.as_mut(), &new_owner, accept_msg)?;

            let actual_owner = ownership::get_ownership(&deps.storage)?.owner;

            assert_that!(&actual_owner).is_equal_to(GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(new_owner),
            });

            Ok(())
        }

        #[test]
        fn updates_governance_type() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            let new_gov = deps.api.addr_make("new_gov");

            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateOwnership(GovAction::TransferOwnership {
                new_owner: GovernanceDetails::Monarchy {
                    monarch: new_gov.to_string(),
                },
                expiry: None,
            });

            execute_as(deps.as_mut(), &owner, msg)?;

            let ownership = ownership::get_ownership(deps.as_ref().storage)?;
            assert_that!(ownership
                .owner
                .owner_address(&deps.as_ref().querier)
                .unwrap()
                .to_string())
            .is_equal_to(owner.to_string());

            let accept_msg = ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership);
            execute_as(deps.as_mut(), &new_gov, accept_msg)?;

            let ownership = ownership::get_ownership(deps.as_ref().storage)?;
            assert_that!(ownership
                .owner
                .owner_address(&deps.as_ref().querier)
                .unwrap()
                .to_string())
            .is_equal_to(new_gov.to_string());

            Ok(())
        }
    }

    mod update_module_addresses {
        use super::*;

        #[test]
        fn manual_adds_module_to_account_modules() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let module1_addr = deps.api.addr_make("module1");
            let module2_addr = deps.api.addr_make("module2");

            mock_init(&mut deps).unwrap();

            let to_add: Vec<(String, String)> = vec![
                ("test:module1".to_string(), module1_addr.to_string()),
                ("test:module2".to_string(), module2_addr.to_string()),
            ];

            let res = update_module_addresses(deps.as_mut(), Some(to_add.clone()), Some(vec![]));
            assert_that!(&res).is_ok();

            let actual_modules = load_account_modules(&deps.storage)?;

            speculoos::prelude::VecAssertions::has_length(
                &mut assert_that!(&actual_modules),
                // Plus proxy
                to_add.len() + 1,
            );
            for (module_id, addr) in to_add {
                speculoos::iter::ContainingIntoIterAssertions::contains(
                    &mut assert_that!(&actual_modules),
                    &(module_id, Addr::unchecked(addr)),
                );
            }

            Ok(())
        }

        #[test]
        fn missing_id() -> ManagerTestResult {
            let mut deps = mock_dependencies();

            mock_init(&mut deps).unwrap();

            let to_add: Vec<(String, String)> = vec![("".to_string(), "module1_addr".to_string())];

            let res = update_module_addresses(deps.as_mut(), Some(to_add), Some(vec![]));
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::InvalidModuleName {});

            Ok(())
        }

        #[test]
        fn manual_removes_module_from_account_modules() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            // manually add module
            ACCOUNT_MODULES.save(
                &mut deps.storage,
                "test:module",
                &Addr::unchecked("test_module_addr"),
            )?;

            let to_remove: Vec<String> = vec!["test:module".to_string()];

            let res = update_module_addresses(deps.as_mut(), Some(vec![]), Some(to_remove));
            assert_that!(&res).is_ok();

            let actual_modules = load_account_modules(&deps.storage)?;

            // Only proxy left
            speculoos::prelude::VecAssertions::has_length(&mut assert_that!(&actual_modules), 1);

            Ok(())
        }

        #[test]
        fn disallows_removing_proxy() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let to_remove: Vec<String> = vec![ACCOUNT.to_string()];

            let res = update_module_addresses(deps.as_mut(), Some(vec![]), Some(to_remove));
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::CannotRemoveProxy {});

            Ok(())
        }

        #[test]
        fn only_account_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            let not_account_factory = deps.api.addr_make("not_account_factory");
            let module_addr = deps.api.addr_make("module_addr");
            mock_init(&mut deps)?;

            // add some thing
            let action_add = InternalConfigAction::UpdateModuleAddresses {
                to_add: Some(vec![("module:other".to_string(), module_addr.to_string())]),
                to_remove: None,
            };
            let msg = ExecuteMsg::UpdateInternalConfig(to_json_binary(&action_add).unwrap());

            // the factory can not call this
            let res = execute_as(deps.as_mut(), &abstr.account_factory, msg.clone());
            assert_that!(&res).is_err();

            // only the owner can
            let res = execute_as(deps.as_mut(), &owner, msg.clone());
            assert_that!(&res).is_ok();

            let res = execute_as(deps.as_mut(), &not_account_factory, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::Ownership(GovOwnershipError::NotOwner));

            Ok(())
        }
    }

    // TODO: move those tests to integrations tests, since we can't do query in unit tests
    mod install_module {
        use super::*;

        #[test]
        fn only_account_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let not_owner = deps.api.addr_make("not_owner");
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::InstallModules {
                modules: vec![ModuleInstallConfig::new(
                    ModuleInfo::from_id_latest("test:module")?,
                    None,
                )],
            };

            let res = execute_as(deps.as_mut(), &not_owner, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::Ownership(GovOwnershipError::NotOwner));

            Ok(())
        }
    }

    mod uninstall_module {
        use std::collections::HashSet;

        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let msg = ExecuteMsg::UninstallModule {
                module_id: "test:module".to_string(),
            };

            test_only_owner(msg)
        }

        #[test]
        fn errors_with_existing_dependents() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            init_with_proxy(&mut deps);

            let test_module = "test:module";
            let msg = ExecuteMsg::UninstallModule {
                module_id: test_module.to_string(),
            };

            // manually add dependents
            let dependents = HashSet::from_iter(vec!["test:dependent".to_string()]);
            DEPENDENTS.save(&mut deps.storage, test_module, &dependents)?;

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::ModuleHasDependents(Vec::from_iter(
                    dependents,
                )));

            Ok(())
        }

        #[test]
        fn disallows_removing_proxy() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UninstallModule {
                module_id: ACCOUNT.to_string(),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::CannotRemoveProxy {});

            Ok(())
        }

        // rest should be in integration tests
    }

    mod exec_on_module {
        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let msg = ExecuteMsg::ExecOnModule {
                module_id: "test:module".to_string(),
                exec_msg: to_json_binary(&"some msg")?,
            };

            test_only_owner(msg)
        }

        #[test]
        fn fails_with_nonexistent_module() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            let missing_module = "test:module".to_string();
            let msg = ExecuteMsg::ExecOnModule {
                module_id: missing_module.clone(),
                exec_msg: to_json_binary(&"some msg")?,
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::ModuleNotFound(missing_module));

            Ok(())
        }

        #[test]
        fn forwards_exec_to_module() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            init_with_proxy(&mut deps);

            let exec_msg = "some msg";

            let msg = ExecuteMsg::ExecOnModule {
                module_id: ACCOUNT.to_string(),
                exec_msg: to_json_binary(&exec_msg)?,
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_ok();

            let msgs = res.unwrap().messages;
            assert_that!(&msgs).has_length(1);

            let expected_msg: CosmosMsg =
                wasm_execute(abstr.account.proxy, &exec_msg, vec![])?.into();

            let actual_msg = &msgs[0];
            assert_that!(&actual_msg.msg).is_equal_to(&expected_msg);

            Ok(())
        }
    }

    mod update_info {
        use abstract_std::objects::validation::ValidationError;

        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: None,
            };

            test_only_owner(msg)
        }
        // integration tests

        #[test]
        fn updates() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            init_with_proxy(&mut deps);

            let name = "new name";
            let description = "new description";
            let link = "http://a.be";

            let msg = ExecuteMsg::UpdateInfo {
                name: Some(name.to_string()),
                description: Some(description.to_string()),
                link: Some(link.to_string()),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_ok();

            let info = INFO.load(deps.as_ref().storage)?;

            assert_that!(&info.name).is_equal_to(name.to_string());
            assert_that!(&info.description.unwrap()).is_equal_to(description.to_string());
            assert_that!(&info.link.unwrap()).is_equal_to(link.to_string());

            Ok(())
        }

        #[test]
        fn removals() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            init_with_proxy(&mut deps);

            let prev_name = "name".to_string();
            INFO.save(
                deps.as_mut().storage,
                &AccountInfo {
                    name: prev_name.clone(),
                    chain_id: "".to_string(),
                    description: Some("description".to_string()),
                    link: Some("link".to_string()),
                },
            )?;

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: None,
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_ok();

            let info = INFO.load(deps.as_ref().storage)?;

            assert_that!(&info.name).is_equal_to(&prev_name);
            assert_that!(&info.description).is_none();
            assert_that!(&info.link).is_none();

            Ok(())
        }

        #[test]
        fn validates_name() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UpdateInfo {
                name: Some("".to_string()),
                description: None,
                link: None,
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    ManagerError::Validation(ValidationError::TitleInvalidShort(_))
                )
            });

            let msg = ExecuteMsg::UpdateInfo {
                name: Some("a".repeat(65)),
                description: None,
                link: None,
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    ManagerError::Validation(ValidationError::TitleInvalidLong(_))
                )
            });

            Ok(())
        }

        #[test]
        fn validates_link() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: Some("aoeu".to_string()),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    ManagerError::Validation(ValidationError::LinkInvalidShort(_))
                )
            });

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: Some("a".repeat(129)),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    ManagerError::Validation(ValidationError::LinkInvalidLong(_))
                )
            });

            Ok(())
        }
    }

    mod handle_callback {
        use super::*;

        #[test]
        fn only_by_contract() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let not_contract = deps.api.addr_make("not_contract");
            mock_init(&mut deps)?;
            let callback = CallbackMsg {};

            let msg = ExecuteMsg::Callback(callback);

            let res = contract::execute(
                deps.as_mut(),
                mock_env(),
                message_info(&not_contract, &[]),
                msg,
            );

            assert_that!(&res)
                .is_err()
                .matches(|err| matches!(err, ManagerError::Std(StdError::GenericErr { .. })));

            Ok(())
        }
    }

    mod update_suspension_status {
        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            test_only_owner(msg)
        }

        #[test]
        fn exec_fails_when_suspended() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(res).is_ok();
            let actual_is_suspended = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that!(&actual_is_suspended).is_true();

            let update_info_msg = ExecuteMsg::UpdateInfo {
                name: Some("asonetuh".to_string()),
                description: None,
                link: None,
            };

            let res = execute_as(deps.as_mut(), &owner, update_info_msg);

            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::AccountSuspended {});

            Ok(())
        }

        #[test]
        fn suspend_account() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);

            assert_that!(&res).is_ok();
            let actual_is_suspended = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that!(&actual_is_suspended).is_true();
            Ok(())
        }

        #[test]
        fn unsuspend_account() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(false),
            };

            let res = execute_as(deps.as_mut(), &owner, msg);

            assert_that!(&res).is_ok();
            let actual_status = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that!(&actual_status).is_false();
            Ok(())
        }
    }

    mod update_internal_config {
        use abstract_std::manager::{InternalConfigAction::UpdateModuleAddresses, QueryMsg};

        use super::*;

        #[test]
        fn only_account_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateInternalConfig(
                to_json_binary(&UpdateModuleAddresses {
                    to_add: None,
                    to_remove: None,
                })
                .unwrap(),
            );

            let bad_sender = deps.api.addr_make("not_account_owner");
            let res = execute_as(deps.as_mut(), &bad_sender, msg.clone());

            assert_that!(&res)
                .is_err()
                .is_equal_to(ManagerError::Ownership(GovOwnershipError::NotOwner));

            let factory_res = execute_as(deps.as_mut(), &abstr.account_factory, msg.clone());
            assert_that!(&factory_res).is_err();

            let owner_res = execute_as(deps.as_mut(), &owner, msg);
            assert_that!(&owner_res).is_ok();

            Ok(())
        }

        #[test]
        fn should_return_err_unrecognized_action() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init(&mut deps)?;

            let msg =
                ExecuteMsg::UpdateInternalConfig(to_json_binary(&QueryMsg::Config {}).unwrap());

            let res = execute_as(deps.as_mut(), &abstr.account_factory, msg);

            assert_that!(&res)
                .is_err()
                .matches(|e| matches!(e, ManagerError::InvalidConfigAction { .. }));

            Ok(())
        }
    }

    mod add_module_upgrade_to_context {
        use super::*;

        #[test]
        fn should_allow_migrate_msg() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let storage = deps.as_mut().storage;

            let result = add_module_upgrade_to_context(storage, TEST_MODULE_ID, vec![]);
            assert_that!(result).is_ok();

            let upgraded_modules: Vec<(String, Vec<Dependency>)> =
                MIGRATE_CONTEXT.load(storage).unwrap();

            assert_that!(upgraded_modules).has_length(1);
            assert_eq!(upgraded_modules[0].0, TEST_MODULE_ID);

            Ok(())
        }
    }

    mod update_ownership {
        use super::*;

        #[test]
        fn allows_ownership_acceptance() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let pending_owner = deps.api.addr_make("not_owner");
            // mock pending owner
            Item::new("ownership").save(
                deps.as_mut().storage,
                &ownership::Ownership {
                    owner: GovernanceDetails::Monarchy { monarch: owner },
                    pending_expiry: None,
                    pending_owner: Some(GovernanceDetails::Monarchy {
                        monarch: pending_owner.clone(),
                    }),
                },
            )?;

            let msg = ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership {});

            execute_as(deps.as_mut(), &pending_owner, msg)?;

            Ok(())
        }
    }

    // upgrade_modules tests are in the integration tests `upgrades`
}
