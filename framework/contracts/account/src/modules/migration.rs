use abstract_std::{
    account::state::{CALLING_TO_AS_ADMIN, CALLING_TO_AS_ADMIN_WILD_CARD},
    adapter::{
        AdapterBaseMsg, AuthorizedAddressesResponse, BaseQueryMsg, QueryMsg as AdapterQuery,
    },
    native_addrs,
    objects::{
        dependency::Dependency,
        module::ModuleInfo,
        module_reference::ModuleReference,
        ownership,
        registry::{RegistryContract, RegistryError},
        storage_namespaces,
    },
    ACCOUNT,
};
use cosmwasm_std::{
    ensure, to_json_binary, Addr, Binary, CosmosMsg, DepsMut, Empty, Env, MessageInfo, Response,
    StdResult, Storage, SubMsg, WasmMsg,
};
use cw2::get_contract_version;
use cw_storage_plus::Item;

use super::{
    _update_whitelisted_modules, configure_adapter, load_module_addr, query_module,
    update_module_addresses,
};
use crate::{
    contract::{AccountResponse, AccountResult, ASSERT_MODULE_DEPENDENCIES_REQUIREMENTS_REPLY_ID},
    error::AccountError,
    queries::query_module_version,
};

pub const MIGRATE_CONTEXT: Item<Vec<(String, Vec<Dependency>)>> =
    Item::new(storage_namespaces::account::MIGRATE_CONTEXT);

/// Migrate modules through address updates or contract migrations
/// The dependency store is updated during migration
/// A reply message is called after performing all the migrations which ensures version compatibility of the new state.
/// Migrations are performed in-order and should be done in a top-down approach.
pub fn upgrade_modules(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    modules: Vec<(ModuleInfo, Option<Binary>)>,
) -> AccountResult {
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;
    ensure!(!modules.is_empty(), AccountError::NoUpdates {});

    let mut upgrade_msgs = vec![];

    let mut account_migrate_info = None;

    let mut upgraded_module_ids = Vec::new();

    CALLING_TO_AS_ADMIN.save(
        deps.storage,
        &Addr::unchecked(CALLING_TO_AS_ADMIN_WILD_CARD),
    )?;

    // Set the migrate messages for each module that's not the account and update the dependency store
    for (module_info, migrate_msg) in modules {
        let module_id = module_info.id();

        // Check for duplicates
        if upgraded_module_ids.contains(&module_id) {
            return Err(AccountError::DuplicateModuleMigration { module_id });
        } else {
            upgraded_module_ids.push(module_id.clone());
        }

        if module_id == ACCOUNT {
            account_migrate_info = Some((module_info, migrate_msg));
        } else {
            set_migrate_msgs_and_context(
                deps.branch(),
                &env,
                module_info,
                migrate_msg,
                &mut upgrade_msgs,
            )?;
        }
    }

    // Upgrade the account last
    if let Some((account_info, account_migrate_msg)) = account_migrate_info {
        upgrade_msgs.push(self_upgrade_msg(
            deps.branch(),
            &env,
            account_info,
            account_migrate_msg.unwrap_or_default(),
        )?);
    }

    let assert_dependency_msg = upgrade_msgs
        .pop()
        .map(|msg| SubMsg::reply_on_success(msg, ASSERT_MODULE_DEPENDENCIES_REQUIREMENTS_REPLY_ID));

    Ok(AccountResponse::new(
        "upgrade_modules",
        vec![("upgraded_modules", upgraded_module_ids.join(","))],
    )
    .add_messages(upgrade_msgs)
    .add_submessages(assert_dependency_msg))
}

pub fn set_migrate_msgs_and_context(
    deps: DepsMut,
    env: &Env,
    module_info: ModuleInfo,
    migrate_msg: Option<Binary>,
    msgs: &mut Vec<CosmosMsg>,
) -> Result<(), AccountError> {
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let registry = RegistryContract::new(deps.as_ref(), abstract_code_id)?;

    let old_module_addr = load_module_addr(deps.storage, &module_info.id())?;
    let old_module_cw2 = query_module_version(deps.as_ref(), old_module_addr.clone(), &registry)?;
    let requested_module = query_module(
        deps.as_ref(),
        env,
        module_info.clone(),
        Some(old_module_cw2),
    )?;

    let migrate_msgs = match requested_module.module.reference {
        // upgrading an adapter is done by moving the authorized addresses to the new contract address and updating the permissions on the account.
        ModuleReference::Adapter(new_adapter_addr) => handle_adapter_migration(
            deps,
            env,
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
        ModuleReference::Standalone(code_id) => {
            vec![build_module_migrate_msg(
                old_module_addr,
                code_id,
                migrate_msg.unwrap(),
            )]
        }
        // Account migrated separately
        _ => return Err(AccountError::NotUpgradeable(module_info)),
    };
    msgs.extend(migrate_msgs);
    Ok(())
}

/// Handle Adapter module migration and return the migration messages
pub fn handle_adapter_migration(
    mut deps: DepsMut,
    env: &Env,
    module_info: ModuleInfo,
    old_adapter_addr: Addr,
    new_adapter_addr: Addr,
) -> AccountResult<Vec<CosmosMsg>> {
    let module_id = module_info.id();
    crate::versioning::assert_migrate_requirements(
        deps.as_ref(),
        &module_id,
        module_info.version.try_into()?,
    )?;
    let old_deps = crate::versioning::load_module_dependencies(deps.as_ref(), &module_id)?;
    // Update the address of the adapter internally
    update_module_addresses(
        deps.branch(),
        vec![(module_id.clone(), new_adapter_addr.clone())],
        Vec::default(),
    )?;

    add_module_upgrade_to_context(deps.storage, &module_id, old_deps)?;

    replace_adapter(deps, env, new_adapter_addr, old_adapter_addr)
}

/// Handle app module migration and return the migration messages
pub fn handle_app_migration(
    deps: DepsMut,
    migrate_msg: Option<Binary>,
    old_module_addr: Addr,
    module_info: ModuleInfo,
    code_id: u64,
) -> AccountResult<Vec<CosmosMsg>> {
    let module_id = module_info.id();
    crate::versioning::assert_migrate_requirements(
        deps.as_ref(),
        &module_id,
        module_info.version.try_into()?,
    )?;
    let old_deps = crate::versioning::load_module_dependencies(deps.as_ref(), &module_id)?;

    // Add module upgrade to reply context
    add_module_upgrade_to_context(deps.storage, &module_id, old_deps)?;

    Ok(vec![build_module_migrate_msg(
        old_module_addr,
        code_id,
        migrate_msg.unwrap_or_else(|| to_json_binary(&Empty {}).unwrap()),
    )])
}

/// Add the module upgrade to the migration context and check for duplicates
pub(crate) fn add_module_upgrade_to_context(
    storage: &mut dyn Storage,
    module_id: &str,
    module_deps: Vec<Dependency>,
) -> Result<(), AccountError> {
    // Add module upgrade to reply context
    let update_context = |mut upgraded_modules: Vec<(String, Vec<Dependency>)>| -> StdResult<Vec<(String, Vec<Dependency>)>> {
        upgraded_modules.push((module_id.to_string(), module_deps));
        Ok(upgraded_modules)
    };
    MIGRATE_CONTEXT.update(storage, update_context)?;

    Ok(())
}

// migrates the module to a new version
pub(crate) fn build_module_migrate_msg(
    module_addr: Addr,
    new_code_id: u64,
    migrate_msg: Binary,
) -> CosmosMsg {
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
    env: &Env,
    new_adapter_addr: Addr,
    old_adapter_addr: Addr,
) -> Result<Vec<CosmosMsg>, AccountError> {
    let mut msgs = vec![];
    // Makes sure we already have the adapter installed
    let AuthorizedAddressesResponse {
        addresses: authorized_addresses,
    } = deps.querier.query_wasm_smart(
        old_adapter_addr.to_string(),
        &<AdapterQuery<Empty>>::Base(BaseQueryMsg::AuthorizedAddresses {
            account_address: env.contract.address.to_string(),
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
    // Replace adapter permissions from old to new address to account
    _update_whitelisted_modules(deps.storage, vec![new_adapter_addr], vec![old_adapter_addr])?;

    Ok(msgs)
}

/// Generate message for upgrading account
///
/// Safety: Account cannot be upgraded to contract that is not confirmed by registry
pub(crate) fn self_upgrade_msg(
    deps: DepsMut,
    env: &Env,
    module_info: ModuleInfo,
    migrate_msg: Binary,
) -> AccountResult<CosmosMsg> {
    let contract = get_contract_version(deps.storage)?;
    let module = query_module(deps.as_ref(), env, module_info.clone(), Some(contract))?;
    if let ModuleReference::Account(account_code_id) = module.module.reference {
        let migration_msg: CosmosMsg<Empty> = CosmosMsg::Wasm(WasmMsg::Migrate {
            contract_addr: env.contract.address.to_string(),
            new_code_id: account_code_id,
            msg: migrate_msg,
        });
        Ok(migration_msg)
    } else {
        Err(AccountError::RegistryError(
            RegistryError::InvalidReference(module_info),
        ))
    }
}

pub fn assert_modules_dependency_requirements(mut deps: DepsMut) -> AccountResult {
    let migrated_modules = MIGRATE_CONTEXT.load(deps.storage)?;

    for (migrated_module_id, old_deps) in migrated_modules {
        crate::versioning::maybe_remove_old_deps(deps.branch(), &migrated_module_id, &old_deps)?;
        let new_deps =
            crate::versioning::maybe_add_new_deps(deps.branch(), &migrated_module_id, &old_deps)?;
        crate::versioning::assert_dependency_requirements(
            deps.as_ref(),
            &new_deps,
            &migrated_module_id,
        )?;
    }

    CALLING_TO_AS_ADMIN.remove(deps.storage);
    MIGRATE_CONTEXT.save(deps.storage, &vec![])?;
    Ok(Response::new())
}
