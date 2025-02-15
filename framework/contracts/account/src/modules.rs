use abstract_std::{
    account::{
        state::{WhitelistedModules, ACCOUNT_ID, ACCOUNT_MODULES, DEPENDENTS, WHITELISTED_MODULES},
        ModuleInstallConfig,
    },
    adapter::{AdapterBaseMsg, BaseExecuteMsg, ExecuteMsg as AdapterExecMsg},
    module_factory::{ExecuteMsg as ModuleFactoryMsg, FactoryModuleInstallConfig},
    native_addrs,
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_factory::ModuleFactoryContract,
        module_reference::ModuleReference,
        ownership,
        registry::RegistryContract,
        salt::generate_instantiate_salt,
        storage_namespaces,
    },
    registry::ModuleResponse,
};
use cosmwasm_std::{
    ensure, wasm_execute, Addr, Attribute, Binary, Coin, CosmosMsg, Deps, DepsMut, Env,
    MessageInfo, StdResult, Storage, SubMsg,
};
use cw2::ContractVersion;
use cw_storage_plus::Item;
use semver::Version;

use crate::{
    contract::{AccountResponse, AccountResult, REGISTER_MODULES_DEPENDENCIES_REPLY_ID},
    error::AccountError,
};

pub use migration::MIGRATE_CONTEXT;
pub(crate) const INSTALL_MODULES_CONTEXT: Item<Vec<(Module, Option<Addr>)>> =
    Item::new(storage_namespaces::account::INSTALL_MODULES_CONTEXT);

pub mod migration;

pub const WHITELIST_SIZE_LIMIT: usize = 15;

/// Attempts to install a new module through the Module Factory Contract
pub fn install_modules(
    mut deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    modules: Vec<ModuleInstallConfig>,
) -> AccountResult {
    // only owner can call this method
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let (install_msgs, install_attribute) = _install_modules(
        deps.branch(),
        modules,
        info.funds, // We forward all the funds to the module_factory address for them to use in the install
        abstract_code_id,
    )?;
    let response = AccountResponse::new("install_modules", std::iter::once(install_attribute))
        .add_submessages(install_msgs);

    Ok(response)
}

/// Generate message and attribute for installing module
/// Adds the modules to the internal store for reference and adds them to the account allowlist if applicable.
pub fn _install_modules(
    mut deps: DepsMut,
    modules: Vec<ModuleInstallConfig>,
    funds: Vec<Coin>,
    abstract_code_id: u64,
) -> AccountResult<(Vec<SubMsg>, Attribute)> {
    let mut installed_modules = Vec::with_capacity(modules.len());
    let mut account_modules = Vec::with_capacity(modules.len());
    let account_id = ACCOUNT_ID.load(deps.storage)?;

    let registry = RegistryContract::new(deps.as_ref(), abstract_code_id)?;
    let module_factory = ModuleFactoryContract::new(deps.as_ref(), abstract_code_id)?;

    let canonical_module_factory = deps
        .api
        .addr_canonicalize(module_factory.address.as_str())?;

    let (infos, init_msgs): (Vec<_>, Vec<_>) =
        modules.into_iter().map(|m| (m.module, m.init_msg)).unzip();
    let modules = registry
        .query_modules_configs(infos, &deps.querier)
        .map_err(|error| AccountError::QueryModulesFailed { error })?;

    let mut install_context = Vec::with_capacity(modules.len());
    let mut add_to_whitelist: Vec<Addr> = Vec::with_capacity(modules.len());
    let mut add_to_account: Vec<(String, Addr)> = Vec::with_capacity(modules.len());

    let salt: Binary = generate_instantiate_salt(&account_id);
    for (ModuleResponse { module, .. }, init_msg) in modules.into_iter().zip(init_msgs) {
        // Check if module is already enabled.
        if ACCOUNT_MODULES.has(deps.storage, &module.info.id()) {
            return Err(AccountError::ModuleAlreadyInstalled(module.info.id()));
        }
        installed_modules.push(module.info.id_with_version());

        let init_msg_salt = match module.reference {
            ModuleReference::Adapter(ref module_address)
            | ModuleReference::Native(ref module_address)
            | ModuleReference::Service(ref module_address) => {
                if module.should_be_whitelisted() {
                    add_to_whitelist.push(module_address.clone());
                }
                add_to_account.push((module.info.id(), module_address.clone()));
                install_context.push((module.clone(), None));
                None
            }
            ModuleReference::App(code_id) | ModuleReference::Standalone(code_id) => {
                let checksum = deps.querier.query_wasm_code_info(code_id)?.checksum;
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
                    add_to_whitelist.push(module_address.clone());
                }
                add_to_account.push((module.info.id(), module_address.clone()));
                install_context.push((module.clone(), Some(module_address)));

                Some(init_msg.ok_or(AccountError::InitMsgMissing(module.info.id()))?)
            }
            _ => return Err(AccountError::ModuleNotInstallable(module.info.to_string())),
        };
        account_modules.push(FactoryModuleInstallConfig::new(module.info, init_msg_salt));
    }
    _update_whitelisted_modules(deps.storage, add_to_whitelist, vec![])?;

    INSTALL_MODULES_CONTEXT.save(deps.storage, &install_context)?;

    let mut messages = vec![];

    // Update module addrs
    update_module_addresses(deps.branch(), add_to_account, vec![])?;

    // Install modules message
    messages.push(SubMsg::reply_on_success(
        wasm_execute(
            module_factory.address,
            &ModuleFactoryMsg::InstallModules {
                modules: account_modules,
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
pub fn update_module_addresses(
    deps: DepsMut,
    to_add: Vec<(String, Addr)>,
    to_remove: Vec<String>,
) -> AccountResult {
    for (id, new_address) in to_add.into_iter() {
        if id.is_empty() {
            return Err(AccountError::InvalidModuleName {});
        };
        // validate addr
        ACCOUNT_MODULES.save(deps.storage, id.as_str(), &new_address)?;
    }

    for id in to_remove.into_iter() {
        ACCOUNT_MODULES.remove(deps.storage, id.as_str());
    }

    Ok(AccountResponse::action("update_module_addresses"))
}

/// Uninstall the module with the ID [`module_id`]
pub fn uninstall_module(
    deps: DepsMut,
    env: &Env,
    info: MessageInfo,
    module_id: String,
) -> AccountResult {
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

    // Remove for account if needed
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let registry = RegistryContract::new(deps.as_ref(), abstract_code_id)?;

    let module = registry.query_module(
        ModuleInfo::from_id(&module_data.module, module_data.version.into())?,
        &deps.querier,
    )?;

    // Remove module from whitelist if it supposed to be removed
    if module.should_be_whitelisted() {
        let module_addr = load_module_addr(deps.storage, &module_id)?;
        _update_whitelisted_modules(deps.storage, vec![], vec![module_addr])?;
    }

    ACCOUNT_MODULES.remove(deps.storage, &module_id);

    let response = AccountResponse::new("uninstall_module", vec![("module", &module_id)]);

    Ok(response)
}

/// Checked load of a module address
pub fn load_module_addr(storage: &dyn Storage, module_id: &str) -> AccountResult<Addr> {
    ACCOUNT_MODULES
        .may_load(storage, module_id)?
        .ok_or_else(|| AccountError::ModuleNotFound(module_id.to_string()))
}

/// Query Registry for the [`Module`] given the provided [`ContractVersion`]
pub fn query_module(
    deps: Deps,
    env: &Env,
    module_info: ModuleInfo,
    old_contract_version: Option<ContractVersion>,
) -> Result<ModuleResponse, AccountError> {
    // Construct feature object to access registry functions
    let abstract_code_id =
        native_addrs::abstract_code_id(&deps.querier, env.contract.address.clone())?;
    let registry = RegistryContract::new(deps, abstract_code_id)?;

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
                reference: registry.query_module_reference_raw(&module_info, &deps.querier)?,
            }
        }
        ModuleVersion::Latest => {
            // Query latest version of contract
            registry.query_module(module_info.clone(), &deps.querier)?
        }
    };

    Ok(ModuleResponse {
        module: Module {
            info: module.info,
            reference: module.reference,
        },
        config: registry.query_config(module_info, &deps.querier)?,
    })
}

#[inline(always)]
fn configure_adapter(
    adapter_address: impl Into<String>,
    message: AdapterBaseMsg,
) -> StdResult<CosmosMsg> {
    let adapter_msg: AdapterExecMsg = BaseExecuteMsg {
        account_address: None,
        msg: message,
    }
    .into();
    Ok(wasm_execute(adapter_address, &adapter_msg, vec![])?.into())
}

/// Update whitelist by adding or removing addresses
/// Note: modules added first before removal, so if module included in both it will be removed
pub(crate) fn _update_whitelisted_modules(
    storage: &mut dyn Storage,
    to_add_module_addresses: Vec<Addr>,
    to_remove_module_addresses: Vec<Addr>,
) -> AccountResult<()> {
    let mut whitelisted_modules = WHITELISTED_MODULES.load(storage)?.0;
    let new_len = (whitelisted_modules.len() + to_add_module_addresses.len())
        .checked_sub(to_remove_module_addresses.len())
        // If overflowed - tried to remove not whitelisted
        .ok_or(AccountError::NotWhitelisted {})?;
    // This is a limit to prevent potentially running out of gas when doing lookups on the modules list
    if new_len > WHITELIST_SIZE_LIMIT {
        return Err(AccountError::ModuleLimitReached {});
    }

    for module_addr in to_add_module_addresses {
        if whitelisted_modules.contains(&module_addr) {
            return Err(AccountError::AlreadyWhitelisted(module_addr.into()));
        }

        // Add contract to whitelist.
        whitelisted_modules.push(module_addr);
    }

    whitelisted_modules.retain(|addr| {
        // retain any addresses that are not in the list of addresses to remove
        !to_remove_module_addresses.contains(addr)
    });

    // Error won't match if something didn't remove
    if whitelisted_modules.len() != new_len {
        return Err(AccountError::NotWhitelisted {});
    }
    WHITELISTED_MODULES.save(storage, &WhitelistedModules(whitelisted_modules))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_only_owner;
    use crate::test_common::{execute_as, mock_init};
    use abstract_std::account::{ExecuteMsg, InternalConfigAction};
    use abstract_std::objects::dependency::Dependency;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr, Order, StdError, Storage};

    fn load_account_modules(storage: &dyn Storage) -> Result<Vec<(String, Addr)>, StdError> {
        ACCOUNT_MODULES
            .range(storage, None, None, Order::Ascending)
            .collect()
    }

    mod add_module_upgrade {

        use crate::modules::migration::add_module_upgrade_to_context;

        use super::*;

        #[coverage_helper::test]
        fn should_allow_migrate_msg() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            mock_init(&mut deps)?;
            let storage = deps.as_mut().storage;

            let result = add_module_upgrade_to_context(storage, TEST_MODULE_ID, vec![]);
            assert!(result.is_ok());

            let upgraded_modules: Vec<(String, Vec<Dependency>)> =
                MIGRATE_CONTEXT.load(storage).unwrap();

            assert_eq!(upgraded_modules.len(), 1);
            assert_eq!(upgraded_modules[0].0, TEST_MODULE_ID);

            Ok(())
        }
    }

    mod update_module_addresses {
        use abstract_std::objects::ownership::GovOwnershipError;

        use super::*;

        #[coverage_helper::test]
        fn manual_adds_module_to_account_modules() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let module1_addr = deps.api.addr_make("module1");
            let module2_addr = deps.api.addr_make("module2");

            mock_init(&mut deps).unwrap();

            let to_add: Vec<(String, Addr)> = vec![
                ("test:module1".to_string(), module1_addr),
                ("test:module2".to_string(), module2_addr),
            ];

            let res = update_module_addresses(deps.as_mut(), to_add.clone(), vec![]);
            assert!(res.is_ok());

            let actual_modules = load_account_modules(&deps.storage)?;

            assert_eq!(actual_modules.len(), to_add.len());
            for (module_id, addr) in to_add {
                assert!(actual_modules.contains(&(module_id, Addr::unchecked(addr))));
            }

            Ok(())
        }

        #[coverage_helper::test]
        fn missing_id() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);

            mock_init(&mut deps).unwrap();

            let to_add: Vec<(String, Addr)> =
                vec![("".to_string(), Addr::unchecked("module1_addr"))];

            let res = update_module_addresses(deps.as_mut(), to_add, vec![]);
            assert_eq!(res, Err(AccountError::InvalidModuleName {}));

            Ok(())
        }

        #[coverage_helper::test]
        fn manual_removes_module_from_account_modules() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            mock_init(&mut deps)?;

            // manually add module
            ACCOUNT_MODULES.save(
                &mut deps.storage,
                "test:module",
                &Addr::unchecked("test_module_addr"),
            )?;

            let to_remove: Vec<String> = vec!["test:module".to_string()];

            let res = update_module_addresses(deps.as_mut(), vec![], to_remove);
            assert!(res.is_ok());

            let actual_modules = load_account_modules(&deps.storage)?;

            assert!(actual_modules.is_empty());

            Ok(())
        }

        #[coverage_helper::test]
        fn only_account_owner() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            let not_account_factory = deps.api.addr_make("not_account_factory");
            let module_addr = deps.api.addr_make("module_addr");
            mock_init(&mut deps)?;

            // add some thing
            let action_add = InternalConfigAction::UpdateModuleAddresses {
                to_add: vec![("module:other".to_string(), module_addr.to_string())],
                to_remove: vec![],
            };
            let msg = ExecuteMsg::UpdateInternalConfig(action_add);

            // the registry can not call this
            let res = execute_as(&mut deps, &abstr.registry, msg.clone());
            assert!(res.is_err());

            // only the owner can
            let res = execute_as(&mut deps, &owner, msg.clone());
            assert!(res.is_ok());

            let res = execute_as(&mut deps, &not_account_factory, msg);
            assert_eq!(
                res,
                Err(AccountError::Ownership(GovOwnershipError::NotOwner))
            );

            Ok(())
        }
    }

    mod uninstall_module {
        use std::collections::HashSet;

        use super::*;

        #[coverage_helper::test]
        fn only_owner() -> anyhow::Result<()> {
            let msg = ExecuteMsg::UninstallModule {
                module_id: "test:module".to_string(),
            };

            test_only_owner(msg)
        }

        #[coverage_helper::test]
        fn errors_with_existing_dependents() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let test_module = "test:module";
            let msg = ExecuteMsg::UninstallModule {
                module_id: test_module.to_string(),
            };

            // manually add dependents
            let dependents = HashSet::from_iter(vec!["test:dependent".to_string()]);
            DEPENDENTS.save(&mut deps.storage, test_module, &dependents)?;

            let res = execute_as(&mut deps, &owner, msg);
            assert_eq!(
                res,
                Err(AccountError::ModuleHasDependents(Vec::from_iter(
                    dependents,
                )))
            );

            Ok(())
        }
    }

    mod exec_on_module {
        use abstract_std::account::ExecuteMsg;
        use cosmwasm_std::to_json_binary;

        use super::*;

        #[coverage_helper::test]
        fn only_owner() -> anyhow::Result<()> {
            let msg = ExecuteMsg::ExecuteOnModule {
                module_id: TEST_MODULE_ID.to_string(),
                exec_msg: to_json_binary(&"some msg")?,
                funds: vec![],
            };

            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let not_owner = deps.api.addr_make("not_owner");
            mock_init(&mut deps)?;

            ACCOUNT_MODULES.save(
                deps.as_mut().storage,
                TEST_MODULE_ID,
                &Addr::unchecked("not-important"),
            )?;

            let res = execute_as(&mut deps, &not_owner, msg);
            assert_eq!(res, Err(AccountError::SenderNotWhitelistedOrOwner {}));
            Ok(())
        }

        #[coverage_helper::test]
        fn fails_with_nonexistent_module() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            let missing_module = "test:module".to_string();
            let msg = ExecuteMsg::ExecuteOnModule {
                module_id: missing_module.clone(),
                exec_msg: to_json_binary(&"some msg")?,
                funds: vec![],
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_eq!(res, Err(AccountError::ModuleNotFound(missing_module)));

            Ok(())
        }

        #[coverage_helper::test]
        fn forwards_exec_to_module() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            deps.querier = abstract_mock_querier(deps.api);
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            update_module_addresses(
                deps.as_mut(),
                vec![("test_mod".to_string(), Addr::unchecked("module_addr"))],
                vec![],
            )?;

            let exec_msg = "some msg";

            let msg = ExecuteMsg::ExecuteOnModule {
                module_id: "test_mod".to_string(),
                exec_msg: to_json_binary(&exec_msg)?,
                funds: vec![],
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert!(res.is_ok());

            let msgs = res.unwrap().messages;
            assert_eq!(msgs.len(), 1);

            let expected_msg: CosmosMsg = wasm_execute("module_addr", &exec_msg, vec![])?.into();

            let actual_msg = &msgs[0];
            assert_eq!(actual_msg.msg, expected_msg);

            Ok(())
        }
    }
}
