use crate::{contract::ManagerResult, error::ManagerError, queries::query_module_cw2};
use crate::{validation, versioning};
use abstract_core::objects::gov_type::GovernanceDetails;
use abstract_core::version_control::ModuleResponse;
use abstract_macros::abstract_response;
use abstract_sdk::{
    core::{
        manager::state::DEPENDENTS,
        manager::state::{
            AccountInfo, SuspensionStatus, ACCOUNT_MODULES, CONFIG, INFO, SUSPENSION_STATUS,
        },
        manager::{CallbackMsg, ExecuteMsg},
        module_factory::ExecuteMsg as ModuleFactoryMsg,
        objects::{
            dependency::Dependency,
            module::{Module, ModuleInfo, ModuleVersion},
            module_reference::ModuleReference,
            validation::{validate_description, validate_link, validate_name},
        },
        proxy::ExecuteMsg as ProxyMsg,
        IBC_CLIENT, MANAGER, PROXY,
    },
    cw_helpers::wasm_smart_query,
    feature_objects::VersionControlContract,
    ModuleRegistryInterface,
};

use abstract_core::adapter::{
    AuthorizedAddressesResponse, BaseExecuteMsg, BaseQueryMsg, ExecuteMsg as AdapterExecMsg,
    QueryMsg as AdapterQuery,
};
use abstract_core::manager::state::ACCOUNT_FACTORY;
use abstract_core::manager::InternalConfigAction;
use abstract_sdk::cw_helpers::AbstractAttributes;
use cosmwasm_std::{
    ensure, from_binary, to_binary, wasm_execute, Addr, Binary, CosmosMsg, Deps, DepsMut, Empty,
    Env, MessageInfo, Response, StdError, StdResult, Storage, WasmMsg,
};
use cw2::{get_contract_version, ContractVersion};
use cw_storage_plus::Item;
use semver::Version;

#[abstract_response(MANAGER)]
pub struct ManagerResponse;

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

// Attempts to install a new module through the Module Factory Contract
pub fn install_module(
    deps: DepsMut,
    msg_info: MessageInfo,
    _env: Env,
    module: ModuleInfo,
    init_msg: Option<Binary>,
) -> ManagerResult {
    // only owner can call this method
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    // Check if module is already enabled.
    if ACCOUNT_MODULES
        .may_load(deps.storage, &module.id())?
        .is_some()
    {
        return Err(ManagerError::ModuleAlreadyInstalled(module.id()));
    }

    let config = CONFIG.load(deps.storage)?;

    let response =
        ManagerResponse::new("install_module", vec![("module", module.id_with_version())])
            .add_message(wasm_execute(
                config.module_factory_address,
                &ModuleFactoryMsg::InstallModule { module, init_msg },
                msg_info.funds, // We forward all the funds to the module_factory address for them to use in the install
            )?);

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
    let proxy_addr = ACCOUNT_MODULES.load(deps.storage, PROXY)?;

    // check if sender is module factory
    if msg_info.sender != config.module_factory_address {
        return Err(ManagerError::CallerNotModuleFactory {});
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
            ..
        } => {
            let id = info.id();
            // assert version requirements
            let dependencies = versioning::assert_install_requirements(deps.as_ref(), &id)?;
            versioning::set_as_dependent(deps.storage, id, dependencies)?;
            response = response.add_message(add_module_to_proxy(
                proxy_addr.into_string(),
                module_address,
            )?)
        }
        Module {
            reference: ModuleReference::Adapter(_),
            info,
            ..
        } => {
            let id = info.id();
            // assert version requirements
            let dependencies = versioning::assert_install_requirements(deps.as_ref(), &id)?;
            versioning::set_as_dependent(deps.storage, id, dependencies)?;
            response = response.add_message(add_module_to_proxy(
                proxy_addr.into_string(),
                module_address,
            )?)
        }
        _ => (),
    };

    Ok(response)
}

/// Execute the [`exec_msg`] on the provided [`module_id`],
pub fn exec_on_module(
    deps: DepsMut,
    msg_info: MessageInfo,
    module_id: String,
    exec_msg: Binary,
) -> ManagerResult {
    // only owner can forward messages to modules
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    let module_addr = load_module_addr(deps.storage, &module_id)?;

    let response = ManagerResponse::new("exec_on_module", vec![("module", module_id)]).add_message(
        CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: module_addr.into(),
            msg: exec_msg,
            funds: vec![],
        }),
    );

    Ok(response)
}

/// Checked load of a module address
fn load_module_addr(storage: &dyn Storage, module_id: &String) -> Result<Addr, ManagerError> {
    ACCOUNT_MODULES
        .may_load(storage, module_id)?
        .ok_or_else(|| ManagerError::ModuleNotFound(module_id.clone()))
}

/// Uninstall the module with the ID [`module_id`]
pub fn uninstall_module(deps: DepsMut, msg_info: MessageInfo, module_id: String) -> ManagerResult {
    // only owner can uninstall modules
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

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
    let module_dependencies = versioning::load_module_dependencies(deps.as_ref(), &module_id)?;
    versioning::remove_as_dependent(deps.storage, &module_id, module_dependencies)?;

    let proxy = ACCOUNT_MODULES.load(deps.storage, PROXY)?;
    let module_addr = load_module_addr(deps.storage, &module_id)?;
    let remove_from_proxy_msg =
        remove_module_from_proxy(proxy.into_string(), module_addr.into_string())?;
    ACCOUNT_MODULES.remove(deps.storage, &module_id);

    Ok(
        ManagerResponse::new("uninstall_module", vec![("module", module_id)])
            .add_message(remove_from_proxy_msg),
    )
}

pub fn set_owner(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    new_owner: GovernanceDetails<String>,
) -> ManagerResult {
    // verify the provided governance details
    let verified_gov = new_owner.verify(deps.api)?;
    let new_owner_addr = verified_gov.owner_address();

    // Update the account information
    let mut acc_info = INFO.load(deps.storage)?;

    // Check that there are changes
    if acc_info.governance_details == verified_gov {
        return Err(ManagerError::NoUpdates {});
    }

    acc_info.governance_details = verified_gov.clone();
    INFO.save(deps.storage, &acc_info)?;

    // Update the Owner of the Account
    let ownership = cw_ownable::update_ownership(
        deps,
        &env.block,
        &info.sender,
        cw_ownable::Action::TransferOwnership {
            new_owner: new_owner_addr.into_string(),
            expiry: None,
        },
    )?;

    let mut attrs = vec![("governance_type", verified_gov.to_string()).into()];
    attrs.extend(ownership.into_attributes());

    Ok(ManagerResponse::new("update_owner", attrs))
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
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
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

        if module_id == MANAGER {
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
            deps,
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

pub fn set_migrate_msgs_and_context(
    deps: DepsMut,
    module_info: ModuleInfo,
    migrate_msg: Option<Binary>,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), ManagerError> {
    let old_module_addr = load_module_addr(deps.storage, &module_info.id())?;
    let old_module_cw2 = query_module_cw2(&deps.as_ref(), old_module_addr.clone())?;
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
        migrate_msg.unwrap_or_else(|| to_binary(&Empty {}).unwrap()),
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
    let proxy_addr = ACCOUNT_MODULES.load(deps.storage, PROXY)?;
    let AuthorizedAddressesResponse {
        addresses: authorized_addresses,
    } = deps.querier.query(&wasm_smart_query(
        old_adapter_addr.to_string(),
        &<AdapterQuery<Empty>>::Base(BaseQueryMsg::AuthorizedAddresses {
            proxy_address: proxy_addr.to_string(),
        }),
    )?)?;
    let authorized_to_migrate: Vec<String> = authorized_addresses
        .into_iter()
        .map(|addr| addr.into_string())
        .collect();
    // Remove authorized addresses from old
    msgs.push(configure_adapter(
        &old_adapter_addr,
        BaseExecuteMsg::UpdateAuthorizedAddresses {
            to_add: vec![],
            to_remove: authorized_to_migrate.clone(),
        },
    )?);
    // Remove adapter as authorized address on dependencies
    msgs.push(configure_adapter(
        &old_adapter_addr,
        BaseExecuteMsg::Remove {},
    )?);
    // Add authorized addresses to new
    msgs.push(configure_adapter(
        &new_adapter_addr,
        BaseExecuteMsg::UpdateAuthorizedAddresses {
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
    msgs.push(add_module_to_proxy(
        proxy_addr.into_string(),
        new_adapter_addr.into_string(),
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
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut info: AccountInfo = INFO.load(deps.storage)?;
    if let Some(name) = name {
        validate_name(&name)?;
        info.name = name;
    }
    validate_description(&description)?;
    info.description = description;
    validate_link(&link)?;
    info.link = link;
    INFO.save(deps.storage, &info)?;

    Ok(ManagerResponse::action("update_info"))
}

pub fn update_suspension_status(
    deps: DepsMut,
    msg_info: MessageInfo,
    is_suspended: SuspensionStatus,
    response: Response,
) -> ManagerResult {
    // only owner can update suspension status
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    SUSPENSION_STATUS.save(deps.storage, &is_suspended)?;

    Ok(response.add_abstract_attributes(vec![("is_suspended", is_suspended.to_string())]))
}

pub fn update_ibc_status(
    deps: DepsMut,
    msg_info: MessageInfo,
    ibc_enabled: bool,
    response: Response,
) -> ManagerResult {
    // only owner can update IBC status
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;
    let proxy = ACCOUNT_MODULES.load(deps.storage, PROXY)?;

    let maybe_client = ACCOUNT_MODULES.may_load(deps.storage, IBC_CLIENT)?;

    let proxy_callback_msg = if ibc_enabled {
        // we have an IBC client so can't add more
        if maybe_client.is_some() {
            return Err(ManagerError::ModuleAlreadyInstalled(IBC_CLIENT.to_string()));
        }

        install_ibc_client(deps, proxy)?
    } else {
        match maybe_client {
            Some(ibc_client) => uninstall_ibc_client(deps, proxy, ibc_client)?,
            None => return Err(ManagerError::ModuleNotFound(IBC_CLIENT.to_string())),
        }
    };

    Ok(response
        .add_abstract_attributes(vec![("ibc_enabled", ibc_enabled.to_string())])
        .add_message(proxy_callback_msg))
}

fn install_ibc_client(deps: DepsMut, proxy: Addr) -> Result<CosmosMsg, ManagerError> {
    // retrieve the latest version
    let ibc_client_module =
        query_module(deps.as_ref(), ModuleInfo::from_id_latest(IBC_CLIENT)?, None)?;

    let ibc_client_addr = ibc_client_module.module.reference.unwrap_native()?;

    ACCOUNT_MODULES.save(deps.storage, IBC_CLIENT, &ibc_client_addr)?;

    Ok(add_module_to_proxy(
        proxy.into_string(),
        ibc_client_addr.to_string(),
    )?)
}

fn uninstall_ibc_client(deps: DepsMut, proxy: Addr, ibc_client: Addr) -> StdResult<CosmosMsg> {
    ACCOUNT_MODULES.remove(deps.storage, IBC_CLIENT);

    remove_module_from_proxy(proxy.into_string(), ibc_client.into_string())
}

/// Query Version Control for the [`Module`] given the provided [`ContractVersion`]
fn query_module(
    deps: Deps,
    module_info: ModuleInfo,
    old_contract_cw2: Option<ContractVersion>,
) -> Result<ModuleResponse, ManagerError> {
    let config = CONFIG.load(deps.storage)?;
    // Construct feature object to access registry functions
    let version_control = VersionControlContract::new(config.version_control_address);
    let version_registry = version_control.module_registry(deps);

    let module = match &module_info.version {
        ModuleVersion::Version(new_version) => {
            let old_contract = old_contract_cw2.unwrap();

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
                reference: version_registry.query_module_reference_raw(&module_info)?,
            }
        }
        ModuleVersion::Latest => {
            // Query latest version of contract
            version_registry.query_module(module_info.clone())?
        }
    };

    Ok(ModuleResponse {
        module: Module {
            info: module.info,
            reference: module.reference,
        },
        config: version_control
            .module_registry(deps)
            .query_all_module_config(module_info)?
            .config,
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

fn add_module_to_proxy(
    proxy_address: String,
    module_address: String,
) -> StdResult<CosmosMsg<Empty>> {
    Ok(wasm_execute(
        proxy_address,
        &ProxyMsg::AddModule {
            module: module_address,
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
    message: BaseExecuteMsg,
) -> StdResult<CosmosMsg> {
    let adapter_msg: AdapterExecMsg<Empty> = message.into();
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

pub fn update_internal_config(deps: DepsMut, info: MessageInfo, config: Binary) -> ManagerResult {
    let action: InternalConfigAction =
        from_binary(&config).map_err(|error| ManagerError::InvalidConfigAction { error })?;
    match action {
        InternalConfigAction::UpdateModuleAddresses { to_add, to_remove } => {
            // only Account Factory/Owner can add custom modules.
            // required to add Proxy after init by Account Factory.
            ACCOUNT_FACTORY
                .assert_admin(deps.as_ref(), &info.sender)
                .or_else(|_| cw_ownable::assert_owner(deps.storage, &info.sender))?;
            update_module_addresses(deps, to_add, to_remove)
        }
        _ => Err(ManagerError::InvalidConfigAction {
            error: StdError::generic_err("Unknown config action"),
        }),
    }
}

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{Order, OwnedDeps, StdError, Storage};

    use crate::contract;
    use speculoos::prelude::*;

    use super::*;
    use crate::test_common::mock_init;

    type ManagerTestResult = Result<(), ManagerError>;

    const TEST_PROXY_ADDR: &str = "proxy";

    fn mock_installed_proxy(deps: DepsMut) -> StdResult<()> {
        let _info = mock_info(TEST_OWNER, &[]);
        ACCOUNT_MODULES.save(deps.storage, PROXY, &Addr::unchecked(TEST_PROXY_ADDR))
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> ManagerResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn _execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> ManagerResult {
        execute_as(deps, TEST_ACCOUNT_FACTORY, msg)
    }

    fn execute_as_owner(deps: DepsMut, msg: ExecuteMsg) -> ManagerResult {
        execute_as(deps, TEST_OWNER, msg)
    }

    fn init_with_proxy(deps: &mut MockDeps) {
        mock_init(deps.as_mut()).unwrap();
        mock_installed_proxy(deps.as_mut()).unwrap();
    }

    fn load_account_modules(storage: &dyn Storage) -> Result<Vec<(String, Addr)>, StdError> {
        ACCOUNT_MODULES
            .range(storage, None, None, Order::Ascending)
            .collect()
    }

    fn test_only_owner(msg: ExecuteMsg) -> ManagerTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let _info = mock_info("not_owner", &[]);

        let res = execute_as(deps.as_mut(), "not_owner", msg);
        assert_that(&res)
            .is_err()
            .is_equal_to(ManagerError::Ownership(OwnershipError::NotOwner));

        Ok(())
    }

    use cw_ownable::OwnershipError;

    type MockDeps = OwnedDeps<MockStorage, MockApi, MockQuerier>;

    mod set_owner_and_gov_type {
        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let msg = ExecuteMsg::SetOwner {
                owner: GovernanceDetails::Monarchy {
                    monarch: "test_owner".to_string(),
                },
            };

            test_only_owner(msg)
        }

        #[test]
        fn validates_new_owner_address() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::SetOwner {
                owner: GovernanceDetails::Monarchy {
                    monarch: "INVALID".to_string(),
                },
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that!(res).is_err().matches(|err| {
                matches!(
                    err,
                    ManagerError::Abstract(abstract_core::AbstractError::Std(
                        StdError::GenericErr { .. }
                    ))
                )
            });
            Ok(())
        }

        #[test]
        fn updates_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_owner = "new_owner";
            let set_owner_msg = ExecuteMsg::SetOwner {
                owner: GovernanceDetails::Monarchy {
                    monarch: new_owner.to_string(),
                },
            };

            let res = execute_as_owner(deps.as_mut(), set_owner_msg);
            assert_that(&res).is_ok();

            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            execute_as(deps.as_mut(), new_owner, accept_msg)?;

            let actual_owner = cw_ownable::get_ownership(&deps.storage)?.owner.unwrap();

            assert_that(&actual_owner).is_equal_to(Addr::unchecked(new_owner));

            Ok(())
        }

        #[test]
        fn updates_governance_type() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_gov = "new_gov".to_string();

            let msg = ExecuteMsg::SetOwner {
                owner: GovernanceDetails::Monarchy { monarch: new_gov },
            };

            execute_as_owner(deps.as_mut(), msg)?;

            let actual_info = INFO.load(deps.as_ref().storage)?;
            assert_that(&actual_info.governance_details.owner_address().to_string())
                .is_equal_to("new_gov".to_string());

            Ok(())
        }
    }

    mod update_module_addresses {
        use super::*;
        use abstract_core::manager::InternalConfigAction;

        #[test]
        fn manual_adds_module_to_account_modules() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let to_add: Vec<(String, String)> = vec![
                ("test:module1".to_string(), "module1_addr".to_string()),
                ("test:module2".to_string(), "module2_addr".to_string()),
            ];

            let res = update_module_addresses(deps.as_mut(), Some(to_add.clone()), Some(vec![]));
            assert_that(&res).is_ok();

            let actual_modules = load_account_modules(&deps.storage)?;

            speculoos::prelude::VecAssertions::has_length(
                &mut assert_that(&actual_modules),
                to_add.len(),
            );
            for (module_id, addr) in to_add {
                speculoos::iter::ContainingIntoIterAssertions::contains(
                    &mut assert_that(&actual_modules),
                    &(module_id, Addr::unchecked(addr)),
                );
            }

            Ok(())
        }

        #[test]
        fn missing_id() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();

            let to_add: Vec<(String, String)> = vec![("".to_string(), "module1_addr".to_string())];

            let res = update_module_addresses(deps.as_mut(), Some(to_add), Some(vec![]));
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::InvalidModuleName {});

            Ok(())
        }

        #[test]
        fn manual_removes_module_from_account_modules() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            // manually add module
            ACCOUNT_MODULES.save(
                &mut deps.storage,
                "test:module",
                &Addr::unchecked("test_module_addr"),
            )?;

            let to_remove: Vec<String> = vec!["test:module".to_string()];

            let res = update_module_addresses(deps.as_mut(), Some(vec![]), Some(to_remove));
            assert_that(&res).is_ok();

            let actual_modules = load_account_modules(&deps.storage)?;

            speculoos::prelude::VecAssertions::is_empty(&mut assert_that(&actual_modules));

            Ok(())
        }

        #[test]
        fn disallows_removing_proxy() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let to_remove: Vec<String> = vec![PROXY.to_string()];

            let res = update_module_addresses(deps.as_mut(), Some(vec![]), Some(to_remove));
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::CannotRemoveProxy {});

            Ok(())
        }

        #[test]
        fn only_account_factory_or_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let action = InternalConfigAction::UpdateModuleAddresses {
                to_add: None,
                to_remove: None,
            };
            let msg = ExecuteMsg::UpdateInternalConfig(to_binary(&action).unwrap());

            let res = execute_as(deps.as_mut(), TEST_ACCOUNT_FACTORY, msg.clone());
            assert_that(&res).is_ok();

            let res = execute_as_owner(deps.as_mut(), msg.clone());
            assert_that(&res).is_ok();

            let res = execute_as(deps.as_mut(), "not_account_factory", msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::Ownership(OwnershipError::NotOwner {}));

            Ok(())
        }
    }

    mod install_module {
        use super::*;

        #[test]
        fn only_account_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id_latest("test:module")?,
                init_msg: None,
            };

            let res = execute_as(deps.as_mut(), "not_owner", msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::Ownership(OwnershipError::NotOwner));

            Ok(())
        }

        #[test]
        fn cannot_reinstall_module() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id_latest("test:module")?,
                init_msg: None,
            };

            // manual installation
            ACCOUNT_MODULES.save(
                &mut deps.storage,
                "test:module",
                &Addr::unchecked("test_module_addr"),
            )?;

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_err().matches(|e| {
                let _module_id = String::from("test:module");
                matches!(e, ManagerError::ModuleAlreadyInstalled(_module_id))
            });

            Ok(())
        }

        #[test]
        fn adds_module_to_account_modules() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::InstallModule {
                module: ModuleInfo::from_id_latest("test:module")?,
                init_msg: None,
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            Ok(())
        }

        #[test]
        fn forwards_init_to_module_factory() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_module = ModuleInfo::from_id_latest("test:module")?;
            let expected_init = Some(to_binary(&"some init msg")?);

            let msg = ExecuteMsg::InstallModule {
                module: new_module.clone(),
                init_msg: expected_init.clone(),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            let msgs = res.unwrap().messages;

            let msg = &msgs[0];

            let expected_msg: CosmosMsg = wasm_execute(
                TEST_MODULE_FACTORY,
                &ModuleFactoryMsg::InstallModule {
                    module: new_module,
                    init_msg: expected_init,
                },
                vec![],
            )?
            .into();
            assert_that(&msgs).has_length(1);

            assert_that(&msg.msg).is_equal_to(&expected_msg);

            Ok(())
        }
    }

    mod uninstall_module {
        use super::*;

        use std::collections::HashSet;

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
            init_with_proxy(&mut deps);

            let test_module = "test:module";
            let msg = ExecuteMsg::UninstallModule {
                module_id: test_module.to_string(),
            };

            // manually add dependents
            let dependents = HashSet::from_iter(vec!["test:dependent".to_string()]);
            DEPENDENTS.save(&mut deps.storage, test_module, &dependents)?;

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::ModuleHasDependents(Vec::from_iter(
                    dependents,
                )));

            Ok(())
        }

        #[test]
        fn disallows_removing_proxy() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UninstallModule {
                module_id: PROXY.to_string(),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::CannotRemoveProxy {});

            Ok(())
        }

        // rest should be in integration tests
    }

    mod register_module {

        use super::*;

        fn _execute_as_module_factory(deps: DepsMut, msg: ExecuteMsg) -> ManagerResult {
            execute_as(deps, TEST_MODULE_FACTORY, msg)
        }

        #[test]
        fn only_module_factory() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let _info = mock_info("not_module_factory", &[]);

            let msg = ExecuteMsg::RegisterModule {
                module_addr: "module_addr".to_string(),
                module: Module {
                    info: ModuleInfo::from_id_latest("test:module")?,
                    reference: ModuleReference::App(1),
                },
            };

            let res = execute_as(deps.as_mut(), "not_module_factory", msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::CallerNotModuleFactory {});

            Ok(())
        }
    }

    mod exec_on_module {
        use super::*;

        #[test]
        fn only_owner() -> ManagerTestResult {
            let msg = ExecuteMsg::ExecOnModule {
                module_id: "test:module".to_string(),
                exec_msg: to_binary(&"some msg")?,
            };

            test_only_owner(msg)
        }

        #[test]
        fn fails_with_nonexistent_module() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let missing_module = "test:module".to_string();
            let msg = ExecuteMsg::ExecOnModule {
                module_id: missing_module.clone(),
                exec_msg: to_binary(&"some msg")?,
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::ModuleNotFound(missing_module));

            Ok(())
        }

        #[test]
        fn forwards_exec_to_module() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let exec_msg = &"some msg";

            let msg = ExecuteMsg::ExecOnModule {
                module_id: PROXY.to_string(),
                exec_msg: to_binary(&exec_msg)?,
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that!(&res).is_ok();

            let msgs = res.unwrap().messages;
            assert_that!(&msgs).has_length(1);

            let expected_msg: CosmosMsg = wasm_execute(TEST_PROXY_ADDR, &exec_msg, vec![])?.into();

            let actual_msg = &msgs[0];
            assert_that!(&actual_msg.msg).is_equal_to(&expected_msg);

            Ok(())
        }
    }

    mod update_info {
        use abstract_core::objects::validation::ValidationError;

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
            init_with_proxy(&mut deps);

            let name = "new name";
            let description = "new description";
            let link = "http://a.be";

            let msg = ExecuteMsg::UpdateInfo {
                name: Some(name.to_string()),
                description: Some(description.to_string()),
                link: Some(link.to_string()),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            let info = INFO.load(deps.as_ref().storage)?;

            assert_that(&info.name).is_equal_to(name.to_string());
            assert_that(&info.description.unwrap()).is_equal_to(description.to_string());
            assert_that(&info.link.unwrap()).is_equal_to(link.to_string());

            Ok(())
        }

        #[test]
        fn removals() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let prev_name = "name".to_string();
            INFO.save(
                deps.as_mut().storage,
                &AccountInfo {
                    name: prev_name.clone(),
                    governance_details: GovernanceDetails::Monarchy {
                        monarch: Addr::unchecked(""),
                    },
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

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            let info = INFO.load(deps.as_ref().storage)?;

            assert_that(&info.name).is_equal_to(&prev_name);
            assert_that(&info.description).is_none();
            assert_that(&info.link).is_none();

            Ok(())
        }

        #[test]
        fn validates_name() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UpdateInfo {
                name: Some("".to_string()),
                description: None,
                link: None,
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_err().matches(|e| {
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

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_err().matches(|e| {
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
            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: Some("aoeu".to_string()),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_err().matches(|e| {
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

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_err().matches(|e| {
                matches!(
                    e,
                    ManagerError::Validation(ValidationError::LinkInvalidLong(_))
                )
            });

            Ok(())
        }
    }

    mod ibc_enabled {
        use super::*;

        const TEST_IBC_CLIENT_ADDR: &str = "ibc_client";

        fn mock_installed_ibc_client(
            deps: &mut OwnedDeps<MockStorage, MockApi, MockQuerier>,
        ) -> StdResult<()> {
            ACCOUNT_MODULES.save(
                &mut deps.storage,
                IBC_CLIENT,
                &Addr::unchecked(TEST_IBC_CLIENT_ADDR),
            )
        }

        #[test]
        fn only_owner() -> ManagerTestResult {
            let msg = ExecuteMsg::UpdateSettings {
                ibc_enabled: Some(true),
            };

            test_only_owner(msg)
        }

        #[test]
        fn throws_if_disabling_without_ibc_client_installed() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            let msg = ExecuteMsg::UpdateSettings {
                ibc_enabled: Some(false),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::ModuleNotFound(IBC_CLIENT.to_string()));

            Ok(())
        }

        #[test]
        fn throws_if_enabling_when_already_enabled() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            mock_installed_ibc_client(&mut deps)?;

            let msg = ExecuteMsg::UpdateSettings {
                ibc_enabled: Some(true),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res)
                .is_err()
                .matches(|e| matches!(e, ManagerError::ModuleAlreadyInstalled(_)));

            Ok(())
        }

        #[test]
        fn uninstall_callback_on_proxy() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            init_with_proxy(&mut deps);

            mock_installed_ibc_client(&mut deps)?;

            let msg = ExecuteMsg::UpdateSettings {
                ibc_enabled: Some(false),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that(&res).is_ok();

            let msgs = res.unwrap().messages;
            assert_that(&msgs).has_length(1);

            let msg = &msgs[0];

            let expected_msg: CosmosMsg = wasm_execute(
                TEST_PROXY_ADDR.to_string(),
                &ProxyMsg::RemoveModule {
                    module: TEST_IBC_CLIENT_ADDR.to_string(),
                },
                vec![],
            )?
            .into();
            assert_that(&msg.msg).is_equal_to(&expected_msg);

            Ok(())
        }

        // integration tests
    }

    mod handle_callback {
        use super::*;

        use cosmwasm_std::StdError;

        #[test]
        fn only_by_contract() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let callback = CallbackMsg {};

            let msg = ExecuteMsg::Callback(callback);

            let res = contract::execute(
                deps.as_mut(),
                mock_env(),
                mock_info("not_contract", &[]),
                msg,
            );

            assert_that(&res)
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
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            test_only_owner(msg)
        }

        #[test]
        fn exec_fails_when_suspended() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            let res = execute_as_owner(deps.as_mut(), msg);
            assert_that!(res).is_ok();
            let actual_is_suspended = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that(&actual_is_suspended).is_true();

            let update_info_msg = ExecuteMsg::UpdateInfo {
                name: Some("asonetuh".to_string()),
                description: None,
                link: None,
            };

            let res = execute_as_owner(deps.as_mut(), update_info_msg);

            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::AccountSuspended {});

            Ok(())
        }

        #[test]
        fn suspend_account() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            let res = execute_as_owner(deps.as_mut(), msg);

            assert_that(&res).is_ok();
            let actual_is_suspended = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that(&actual_is_suspended).is_true();
            Ok(())
        }

        #[test]
        fn unsuspend_account() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(false),
            };

            let res = execute_as_owner(deps.as_mut(), msg);

            assert_that(&res).is_ok();
            let actual_status = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that(&actual_status).is_false();
            Ok(())
        }
    }

    mod update_internal_config {
        use super::*;
        use abstract_core::manager::InternalConfigAction::UpdateModuleAddresses;
        use abstract_core::manager::QueryMsg;

        #[test]
        fn only_account_factory_or_owner() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateInternalConfig(
                to_binary(&UpdateModuleAddresses {
                    to_add: None,
                    to_remove: None,
                })
                .unwrap(),
            );

            let bad_sender = "not_account_factory";
            let res = execute_as(deps.as_mut(), bad_sender, msg.clone());

            assert_that(&res)
                .is_err()
                .is_equal_to(ManagerError::Ownership(OwnershipError::NotOwner {}));

            let owner_res = execute_as_owner(deps.as_mut(), msg.clone());
            assert_that(&owner_res).is_ok();

            let factory_res = execute_as(deps.as_mut(), TEST_ACCOUNT_FACTORY, msg);
            assert_that(&factory_res).is_ok();

            Ok(())
        }

        #[test]
        fn should_return_err_unrecognized_action() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateInternalConfig(to_binary(&QueryMsg::Config {}).unwrap());

            let res = execute_as(deps.as_mut(), TEST_ACCOUNT_FACTORY, msg);

            assert_that(&res)
                .is_err()
                .matches(|e| matches!(e, ManagerError::InvalidConfigAction { .. }));

            Ok(())
        }
    }

    mod add_module_upgrade_to_context {
        use super::*;
        use abstract_testing::prelude::TEST_MODULE_ID;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn should_allow_migrate_msg() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
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
            mock_init(deps.as_mut())?;

            let pending_owner = "not_owner";
            // mock pending owner
            Item::new("ownership").save(
                deps.as_mut().storage,
                &cw_ownable::Ownership {
                    owner: None,
                    pending_expiry: None,
                    pending_owner: Some(Addr::unchecked(pending_owner)),
                },
            )?;

            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership {});

            execute_as(deps.as_mut(), pending_owner, msg)?;

            Ok(())
        }

        #[test]
        fn allows_renouncing() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::RenounceOwnership {});

            execute_as_owner(deps.as_mut(), msg)?;

            Ok(())
        }

        #[test]
        fn disallows_ownership_transfer() -> ManagerTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let transfer_to = "not_owner";

            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: transfer_to.to_string(),
                expiry: None,
            });

            let res = execute_as_owner(deps.as_mut(), msg);

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::MustUseSetOwner {});

            Ok(())
        }
    }

    // upgrade_modules tests are in the integration tests `upgrades`
}
