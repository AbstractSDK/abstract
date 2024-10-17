use abstract_sdk::std::{
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        namespace::Namespace,
        AccountId,
    },
    registry::{
        state::{ACCOUNT_ADDRESSES, REGISTERED_MODULES, YANKED_MODULES},
        ModuleFilter, ModuleResponse, ModulesListResponse, ModulesResponse, NamespaceListResponse,
    },
};
use abstract_std::{
    objects::module::ModuleStatus,
    registry::{
        state::{NAMESPACES, PENDING_MODULES, REV_NAMESPACES},
        AccountListResponse, AccountsResponse, ModuleConfiguration, NamespaceInfo,
        NamespaceResponse,
    },
};
use cosmwasm_std::{Deps, Order, StdError, StdResult};
use cw_storage_plus::{Bound, Map};

use crate::{contract::VCResult, error::RegistryError};

const DEFAULT_LIMIT: u8 = 10;
const MAX_LIMIT: u8 = 20;

pub fn handle_accounts_address_query(
    deps: Deps,
    account_ids: Vec<AccountId>,
) -> StdResult<AccountsResponse> {
    let account_address = account_ids
        .into_iter()
        .map(|account_id| {
            ACCOUNT_ADDRESSES
                .load(deps.storage, &account_id)
                .map_err(|_| {
                    StdError::generic_err(
                        RegistryError::UnknownAccountId { id: account_id }.to_string(),
                    )
                })
        })
        .collect::<Result<Vec<_>, _>>()?;

    Ok(AccountsResponse {
        accounts: account_address,
    })
}

pub fn handle_modules_query(deps: Deps, modules: Vec<ModuleInfo>) -> StdResult<ModulesResponse> {
    let mut modules_response = ModulesResponse { modules: vec![] };
    for mut module in modules {
        let maybe_module_ref = if let ModuleVersion::Version(_) = module.version {
            REGISTERED_MODULES.load(deps.storage, &module)
        } else {
            // get latest
            let versions: StdResult<Vec<(String, ModuleReference)>> = REGISTERED_MODULES
                .prefix((module.namespace.clone(), module.name.clone()))
                .range(deps.storage, None, None, Order::Descending)
                .take(1)
                .collect();
            let (latest_version, id) = versions?
                .first()
                .ok_or_else(|| {
                    StdError::generic_err(RegistryError::ModuleNotFound(module.clone()).to_string())
                })?
                .clone();
            module.version = ModuleVersion::Version(latest_version);
            Ok(id)
        };

        match maybe_module_ref {
            Err(_) => Err(StdError::generic_err(
                RegistryError::ModuleNotFound(module).to_string(),
            )),
            Ok(mod_ref) => {
                modules_response.modules.push(ModuleResponse {
                    module: Module {
                        info: module.clone(),
                        reference: mod_ref,
                    },
                    config: ModuleConfiguration::from_storage(deps.storage, &module)?,
                });
                Ok(())
            }
        }?;
    }

    Ok(modules_response)
}

pub fn handle_account_list_query(
    deps: Deps,
    start_after: Option<AccountId>,
    limit: Option<u8>,
) -> VCResult<AccountListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let start_bound = start_after.as_ref().map(Bound::exclusive);

    // Load all accounts
    let accounts = ACCOUNT_ADDRESSES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(AccountListResponse { accounts })
}

pub fn handle_module_list_query(
    deps: Deps,
    start_after: Option<ModuleInfo>,
    limit: Option<u8>,
    filter: Option<ModuleFilter>,
) -> VCResult<ModulesListResponse> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let ModuleFilter {
        namespace: ref namespace_filter,
        name: ref name_filter,
        version: version_filter,
        status,
    } = filter.unwrap_or_default();

    let mod_lib = match status {
        Some(ModuleStatus::Registered) => REGISTERED_MODULES,
        Some(ModuleStatus::Pending) => PENDING_MODULES,
        Some(ModuleStatus::Yanked) => YANKED_MODULES,
        None => REGISTERED_MODULES,
    };
    let mut modules: Vec<(ModuleInfo, ModuleReference)> = vec![];

    if let Some(namespace_filter) = namespace_filter {
        let namespace_filter = Namespace::new(namespace_filter)?;
        modules.extend(filter_modules_by_namespace(
            deps,
            start_after,
            limit,
            namespace_filter,
            name_filter,
            mod_lib,
        )?);
    } else {
        let start_bound: Option<Bound<&ModuleInfo>> = start_after.as_ref().map(Bound::exclusive);

        // Load all modules
        modules.extend(
            mod_lib
                .range(deps.storage, start_bound, None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?,
        );
    };

    // handle name and version filter after loading all modules
    if namespace_filter.is_none() && name_filter.is_some() {
        let name_filter = name_filter.as_ref().unwrap();
        modules.retain(|(module_info, _)| &module_info.name == name_filter);
    }
    if let Some(version) = version_filter.map(ModuleVersion::Version) {
        modules.retain(|(info, _)| info.version == version);
    }

    let modules = modules
        .into_iter()
        .map(|(module_info, mod_ref)| {
            Ok(ModuleResponse {
                module: Module {
                    info: module_info.clone(),
                    reference: mod_ref,
                },
                config: ModuleConfiguration::from_storage(deps.storage, &module_info)?,
            })
        })
        .collect::<Result<Vec<_>, StdError>>()?;

    Ok(ModulesListResponse { modules })
}

pub fn handle_namespaces_query(
    deps: Deps,
    accounts: Vec<AccountId>,
) -> StdResult<NamespaceListResponse> {
    let namespaces = accounts
        .into_iter()
        .filter_map(|account_id| {
            REV_NAMESPACES
                .may_load(deps.storage, &account_id)
                .transpose()
                .map(|namespace_res| namespace_res.map(|namespace| (namespace, account_id)))
        })
        .collect::<StdResult<Vec<_>>>()?;
    Ok(NamespaceListResponse { namespaces })
}

pub fn handle_namespace_query(deps: Deps, namespace: Namespace) -> StdResult<NamespaceResponse> {
    let account_id = NAMESPACES.may_load(deps.storage, &namespace)?;
    let Some(account_id) = account_id else {
        return Ok(NamespaceResponse::Unclaimed {});
    };

    let account = ACCOUNT_ADDRESSES.load(deps.storage, &account_id)?;
    Ok(NamespaceResponse::Claimed(NamespaceInfo {
        account_id,
        account,
    }))
}

pub fn handle_namespace_list_query(
    deps: Deps,
    start_after: Option<Namespace>,
    limit: Option<u8>,
) -> StdResult<NamespaceListResponse> {
    let start_bound = start_after.as_ref().map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let namespaces = NAMESPACES
        .range(deps.storage, start_bound, None, Order::Ascending)
        .take(limit)
        .collect::<StdResult<Vec<_>>>()?;

    Ok(NamespaceListResponse { namespaces })
}

/// Filter the modules with their primary key prefix (namespace)
fn filter_modules_by_namespace(
    deps: Deps,
    start_after: Option<ModuleInfo>,
    limit: usize,
    namespace: Namespace,
    name: &Option<String>,
    mod_lib: Map<&ModuleInfo, ModuleReference>,
) -> StdResult<Vec<(ModuleInfo, ModuleReference)>> {
    let mut modules: Vec<(ModuleInfo, ModuleReference)> = vec![];

    // Filter by name using full prefix
    if let Some(name) = name {
        let start_bound: Option<Bound<String>> =
            start_after.map(|info| Bound::exclusive(info.version.to_string()));

        modules.extend(
            mod_lib
                .prefix((namespace.clone(), name.clone()))
                .range(deps.storage, start_bound, None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?
                .into_iter()
                .map(|(version, reference)| {
                    (
                        ModuleInfo {
                            namespace: namespace.clone(),
                            name: name.clone(),
                            version: ModuleVersion::Version(version),
                        },
                        reference,
                    )
                }),
        )
    } else {
        // Filter by just namespace using sub prefix
        let start_bound: Option<Bound<(String, String)>> =
            start_after.map(|token| Bound::exclusive((token.name, token.version.to_string())));

        modules.extend(
            mod_lib
                .sub_prefix(namespace.clone())
                .range(deps.storage, start_bound, None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?
                .into_iter()
                .map(|((name, version), reference)| {
                    (
                        ModuleInfo {
                            namespace: namespace.clone(),
                            name,
                            version: ModuleVersion::Version(version),
                        },
                        reference,
                    )
                }),
        );
    }
    Ok(modules)
}

#[cfg(test)]
mod test {
    #![allow(clippy::needless_borrows_for_generic_args)]
    use super::*;

    use crate::contract;
    use abstract_std::{account, objects::account::AccountTrace, registry::*};
    use abstract_testing::{prelude::*, MockQuerierOwnership};
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies, MockApi},
        Addr, Binary, StdError,
    };

    type RegistryTestResult = Result<(), RegistryError>;

    const TEST_OTHER: &str = "testother";
    const TEST_OTHER_ACCOUNT_ID: AccountId = AccountId::const_new(2, AccountTrace::Local);
    const TEST_OTHER_ACCOUNT_ADDR: &str = "account1";

    pub fn mock_account_querier(mock_api: MockApi) -> MockQuerierBuilder {
        let abstr = AbstractMockAddrs::new(mock_api);
        let account = test_account(mock_api);
        let other_account = mock_api.addr_make(TEST_OTHER_ACCOUNT_ADDR);
        let other_owner = mock_api.addr_make(TEST_OTHER);
        MockQuerierBuilder::default()
            .with_smart_handler(account.addr(), move |msg| {
                let abstr = AbstractMockAddrs::new(mock_api);
                match from_json(msg).unwrap() {
                    account::QueryMsg::Config {} => {
                        let resp = account::ConfigResponse {
                            registry_address: abstr.registry,
                            module_factory_address: abstr.module_factory,
                            account_id: TEST_ACCOUNT_ID, // mock value, not used
                            is_suspended: false,
                            whitelisted_addresses: vec![],
                        };
                        Ok(to_json_binary(&resp).unwrap())
                    }
                    _ => panic!("unexpected message"),
                }
            })
            .with_contract_item(
                account.addr(),
                account::state::ACCOUNT_ID,
                &AccountId::local(1),
            )
            .with_contract_item(
                account.addr(),
                cw2::CONTRACT,
                &cw2::ContractVersion {
                    contract: abstract_std::ACCOUNT.to_owned(),
                    version: contract::CONTRACT_VERSION.to_owned(),
                },
            )
            .with_smart_handler(&other_account, move |msg| {
                match from_json(msg).unwrap() {
                    account::QueryMsg::Config {} => {
                        let abstr = AbstractMockAddrs::new(mock_api);
                        let resp = account::ConfigResponse {
                            registry_address: abstr.registry,
                            module_factory_address: abstr.module_factory,
                            account_id: TEST_OTHER_ACCOUNT_ID, // mock value, not used
                            is_suspended: false,
                            whitelisted_addresses: vec![],
                        };
                        Ok(to_json_binary(&resp).unwrap())
                    }
                    _ => panic!("unexpected message"),
                }
            })
            .with_contract_item(
                &other_account,
                account::state::ACCOUNT_ID,
                &AccountId::local(2),
            )
            .with_contract_item(
                &other_account,
                cw2::CONTRACT,
                &cw2::ContractVersion {
                    contract: abstract_std::ACCOUNT.to_owned(),
                    version: contract::CONTRACT_VERSION.to_owned(),
                },
            )
            .with_owner(account.addr(), Some(&abstr.owner))
            .with_owner(&other_account, Some(&other_owner))
    }

    fn mock_init(deps: &mut MockDeps) -> RegistryTestResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let info = message_info(&abstr.owner, &[]);
        let env = mock_env_validated(deps.api);
        let admin = info.sender.to_string();

        contract::instantiate(
            deps.as_mut(),
            env,
            info,
            InstantiateMsg {
                admin,
                security_disabled: Some(true),
                namespace_registration_fee: None,
            },
        )?;

        Ok(())
    }

    /// Initialize the registry with admin as creator and test account
    fn mock_init_with_account(deps: &mut MockDeps) -> VCResult {
        let abstr = AbstractMockAddrs::new(deps.api);
        let account = test_account(deps.api);
        mock_init(deps)?;

        state::REGISTERED_MODULES.save(
            &mut deps.storage,
            &ModuleInfo::from_id(abstract_std::ACCOUNT, contract::CONTRACT_VERSION.into()).unwrap(),
            &ModuleReference::Account(1),
        )?;
        // 0 occupied
        state::LOCAL_ACCOUNT_SEQUENCE.save(&mut deps.storage, &1)?;

        execute_as(
            deps,
            account.addr(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.clone().into(),
            },
        )?;

        let other_account = Account::new(deps.api.addr_make(TEST_OTHER_ACCOUNT_ADDR));
        execute_as(
            deps,
            other_account.addr(),
            ExecuteMsg::AddAccount {
                namespace: None,
                creator: abstr.owner.into(),
            },
        )
    }

    fn execute_as(deps: &mut MockDeps, sender: &Addr, msg: ExecuteMsg) -> VCResult {
        let env = mock_env_validated(deps.api);
        contract::execute(deps.as_mut(), env, message_info(sender, &[]), msg)
    }

    fn query_helper(deps: &MockDeps, msg: QueryMsg) -> VCResult<Binary> {
        contract::query(deps.as_ref(), mock_env_validated(deps.api), msg)
    }

    mod module {
        use super::*;

        use abstract_std::objects::module::ModuleVersion::Latest;

        fn add_namespace(deps: &mut MockDeps, namespace: &str) {
            let abstr = AbstractMockAddrs::new(deps.api);
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: TEST_ACCOUNT_ID,
                namespace: namespace.to_string(),
            };

            let res = execute_as(deps, &abstr.owner, msg);
            assert!(res.is_ok());
        }

        fn add_module(deps: &mut MockDeps, new_module_info: ModuleInfo) {
            let abstr = AbstractMockAddrs::new(deps.api);

            let add_msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module_info, ModuleReference::App(0))],
            };

            let res = execute_as(deps, &abstr.owner, add_msg);
            assert!(res.is_ok());
        }

        #[coverage_helper::test]
        fn get_module() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            mock_init_with_account(&mut deps)?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo {
                name, namespace, ..
            } = new_module_info.clone();

            add_namespace(&mut deps, "test");
            add_module(&mut deps, new_module_info.clone());

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace,
                    name,
                    version: Latest {},
                }],
            };

            let ModulesResponse { mut modules } = from_json(query_helper(&deps, query_msg)?)?;
            assert_eq!(modules.swap_remove(0).module.info, new_module_info);
            Ok(())
        }

        #[coverage_helper::test]
        fn none_when_no_matching_version() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            mock_init_with_account(&mut deps)?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo {
                name, namespace, ..
            } = new_module_info.clone();

            add_namespace(&mut deps, "test");
            add_module(&mut deps, new_module_info);

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace,
                    name,
                    version: ModuleVersion::Version("024209.902.902".to_string()),
                }],
            };

            let res = query_helper(&deps, query_msg);
            assert!(matches!(
                res,
                Err(RegistryError::Std(StdError::GenericErr { .. }))
            ));
            Ok(())
        }

        #[coverage_helper::test]
        fn get_latest_when_multiple_registered() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            mock_init_with_account(&mut deps)?;

            add_namespace(&mut deps, "test");

            let module_id = "test:module";
            let oldest_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("0.1.2".into())).unwrap();

            add_module(&mut deps, oldest_version);
            let newest_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("100.1.2".into())).unwrap();
            add_module(&mut deps, newest_version.clone());

            let another_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("1.1.2".into())).unwrap();
            add_module(&mut deps, another_version);

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace: Namespace::new("test")?,
                    name: "module".to_string(),
                    version: Latest {},
                }],
            };

            let ModulesResponse { mut modules } = from_json(query_helper(&deps, query_msg)?)?;
            assert_eq!(modules.swap_remove(0).module.info, newest_version);
            Ok(())
        }
    }

    use cosmwasm_std::from_json;

    /// Add namespaces
    fn add_namespaces(
        deps: &mut MockDeps,
        acc_and_namespace: Vec<(AccountId, &str)>,
        sender: &Addr,
    ) {
        for (account_id, namespace) in acc_and_namespace {
            let msg = ExecuteMsg::ClaimNamespace {
                account_id,
                namespace: namespace.to_string(),
            };

            let res = execute_as(deps, sender, msg);
            assert!(res.is_ok());
        }
    }

    /// Add the provided modules to the registry
    fn propose_modules(deps: &mut MockDeps, new_module_infos: Vec<ModuleInfo>, sender: &Addr) {
        let modules = new_module_infos
            .into_iter()
            .map(|info| (info, ModuleReference::App(0)))
            .collect();
        let add_msg = ExecuteMsg::ProposeModules { modules };
        let res = execute_as(deps, sender, add_msg);
        assert!(res.is_ok());
    }

    /// Yank the provided module in the registry
    fn yank_module(deps: &mut MockDeps, module_info: ModuleInfo) {
        let abstr = AbstractMockAddrs::new(deps.api);
        let yank_msg = ExecuteMsg::YankModule {
            module: module_info,
        };
        let res = execute_as(deps, &abstr.owner, yank_msg);
        assert!(res.is_ok());
    }

    /// Init verison control with some test modules.
    fn init_with_mods(deps: &mut MockDeps) {
        let abstr = AbstractMockAddrs::new(deps.api);
        mock_init_with_account(deps).unwrap();

        add_namespaces(deps, vec![(TEST_ACCOUNT_ID, "cw-plus")], &abstr.owner);
        let other = deps.api.addr_make(TEST_OTHER);
        add_namespaces(deps, vec![(TEST_OTHER_ACCOUNT_ID, "4t2")], &other);

        let cw_mods = vec![
            ModuleInfo::from_id("cw-plus:module1", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("cw-plus:module2", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("cw-plus:module3", ModuleVersion::Version("0.1.2".into())).unwrap(),
        ];
        propose_modules(deps, cw_mods, &abstr.owner);

        let fortytwo_mods = vec![
            ModuleInfo::from_id("4t2:module1", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("4t2:module2", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("4t2:module3", ModuleVersion::Version("0.1.2".into())).unwrap(),
        ];
        propose_modules(deps, fortytwo_mods, &other);
    }

    mod modules {
        use super::*;

        #[coverage_helper::test]
        fn get_cw_plus_modules() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let namespace = Namespace::new("cw-plus")?;

            let query_msg = QueryMsg::Modules {
                infos: vec![
                    ModuleInfo {
                        namespace: namespace.clone(),
                        name: "module1".to_string(),
                        version: ModuleVersion::Latest {},
                    },
                    ModuleInfo {
                        namespace: namespace.clone(),
                        name: "module2".to_string(),
                        version: ModuleVersion::Latest {},
                    },
                    ModuleInfo {
                        namespace: namespace.clone(),
                        name: "module3".to_string(),
                        version: ModuleVersion::Latest {},
                    },
                ],
            };

            let ModulesResponse { modules } = from_json(query_helper(&deps, query_msg)?)?;
            assert_eq!(modules.len(), 3);
            for module in modules {
                assert_eq!(module.module.info.namespace, namespace.clone());
                assert_eq!(
                    module.module.info.version,
                    ModuleVersion::Version("0.1.2".into())
                );
            }
            Ok(())
        }

        #[coverage_helper::test]
        fn get_modules_not_found() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace: Namespace::new("not")?,
                    name: "found".to_string(),
                    version: ModuleVersion::Latest {},
                }],
            };

            let res = query_helper(&deps, query_msg);
            assert!(matches!(
                res,
                Err(RegistryError::Std(StdError::GenericErr { .. }))
            ));
            Ok(())
        }
    }

    mod list_modules {
        use super::*;

        fn filtered_list_msg(filter: ModuleFilter) -> QueryMsg {
            QueryMsg::ModuleList {
                filter: Some(filter),
                start_after: None,
                limit: None,
            }
        }

        #[coverage_helper::test]
        fn filter_by_namespace_existing() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);
            let filtered_namespace = "cw-plus".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                ..Default::default()
            };
            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 3);
            for entry in modules {
                assert_eq!(
                    entry.module.info.namespace,
                    Namespace::unchecked(filtered_namespace.clone())
                );
            }
        }

        #[coverage_helper::test]
        fn filter_default_returns_only_non_yanked() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            let abstr = AbstractMockAddrs::new(deps.api);
            init_with_mods(&mut deps);

            let cw_mods = vec![
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            ];
            propose_modules(&mut deps, cw_mods, &abstr.owner);
            yank_module(
                &mut deps,
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );
            yank_module(
                &mut deps,
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );

            let list_msg = QueryMsg::ModuleList {
                filter: None,
                start_after: None,
                limit: None,
            };

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 7);
            let yanked_module_names = ["module4".to_string(), "module5".to_string()];
            for entry in modules {
                if entry.module.info.namespace == Namespace::unchecked("cw-plus") {
                    assert!(!yanked_module_names
                        .iter()
                        .any(|e| e == &entry.module.info.name));
                }
            }
        }

        #[coverage_helper::test]
        fn filter_yanked_by_namespace_existing() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            let abstr = AbstractMockAddrs::new(deps.api);
            init_with_mods(&mut deps);

            let cw_mods = vec![
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            ];
            propose_modules(&mut deps, cw_mods, &abstr.owner);
            yank_module(
                &mut deps,
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );
            yank_module(
                &mut deps,
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );

            let filtered_namespace = "cw-plus".to_string();

            let filter = ModuleFilter {
                status: Some(ModuleStatus::Yanked),
                namespace: Some(filtered_namespace.clone()),
                ..Default::default()
            };
            let list_msg = QueryMsg::ModuleList {
                filter: Some(filter),
                start_after: None,
                limit: None,
            };

            let res = query_helper(&deps, list_msg);
            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 2);

            for entry in modules {
                assert_eq!(
                    entry.module.info.namespace,
                    Namespace::unchecked(filtered_namespace.clone())
                );
            }
        }

        #[coverage_helper::test]
        fn filter_by_namespace_non_existing() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            let abstr = AbstractMockAddrs::new(deps.api);
            mock_init_with_account(&mut deps).unwrap();
            add_namespaces(&mut deps, vec![(TEST_ACCOUNT_ID, "cw-plus")], &abstr.owner);
            let cw_mods = vec![ModuleInfo::from_id(
                "cw-plus:module1",
                ModuleVersion::Version("0.1.2".into()),
            )
            .unwrap()];
            propose_modules(&mut deps, cw_mods, &abstr.owner);

            let filtered_namespace = "cw-plus".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 1);
        }

        #[coverage_helper::test]
        fn filter_by_namespace_and_name() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let filtered_namespace = "cw-plus".to_string();
            let filtered_name = "module2".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                name: Some(filtered_name.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 1);

            let module = modules[0].clone();
            assert_eq!(
                module.module.info.namespace,
                Namespace::unchecked(filtered_namespace.clone())
            );
            assert_eq!(module.module.info.name, filtered_name.clone());
        }

        #[coverage_helper::test]
        fn filter_by_namespace_and_name_with_multiple_versions() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            let abstr = AbstractMockAddrs::new(deps.api);
            init_with_mods(&mut deps);

            let filtered_namespace = "cw-plus".to_string();
            let filtered_name = "module2".to_string();

            propose_modules(
                &mut deps,
                vec![ModuleInfo::from_id(
                    format!("{filtered_namespace}:{filtered_name}").as_str(),
                    ModuleVersion::Version("0.1.3".into()),
                )
                .unwrap()],
                &abstr.owner,
            );

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                name: Some(filtered_name.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 2);

            for module in modules {
                assert_eq!(
                    module.module.info.namespace,
                    Namespace::unchecked(filtered_namespace.clone())
                );
                assert_eq!(module.module.info.name, filtered_name.clone());
            }
        }

        #[coverage_helper::test]
        fn filter_by_only_version_many() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let filtered_version = "0.1.2".to_string();

            let filter = ModuleFilter {
                version: Some(filtered_version.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 6);

            for module in modules {
                assert_eq!(
                    module.module.info.version.to_string(),
                    filtered_version.clone()
                );
            }
        }

        #[coverage_helper::test]
        fn filter_by_only_version_none() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let filtered_version = "5555".to_string();

            let filter = ModuleFilter {
                version: Some(filtered_version),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert!(modules.is_empty());
        }

        #[coverage_helper::test]
        fn filter_by_name_and_version() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let filtered_name = "module2".to_string();
            let filtered_version = "0.1.2".to_string();

            let filter = ModuleFilter {
                name: Some(filtered_name.clone()),
                version: Some(filtered_version.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            // We expect two because both cw-plus and snth have a module2 with version 0.1.2
            assert_eq!(modules.len(), 2);

            for module in modules {
                assert_eq!(module.module.info.name, filtered_name.clone());
                assert_eq!(
                    module.module.info.version.to_string(),
                    filtered_version.clone()
                );
            }
        }

        #[coverage_helper::test]
        fn filter_by_namespace_and_version() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            let filtered_namespace = "cw-plus".to_string();
            let filtered_version = "0.1.2".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                version: Some(filtered_version.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(&deps, list_msg);

            let ModulesListResponse { modules } = from_json(res.unwrap()).unwrap();
            assert_eq!(modules.len(), 3);

            for module in modules {
                assert_eq!(
                    module.module.info.namespace,
                    Namespace::unchecked(filtered_namespace.clone())
                );
                assert_eq!(
                    module.module.info.version.to_string(),
                    filtered_version.clone()
                );
            }
        }
    }

    mod query_namespaces {
        use super::*;

        #[coverage_helper::test]
        fn namespaces() {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            init_with_mods(&mut deps);

            // get for test other account
            let res = query_helper(
                &deps,
                QueryMsg::Namespaces {
                    accounts: vec![TEST_OTHER_ACCOUNT_ID],
                },
            );
            let NamespacesResponse { namespaces } = from_json(res.unwrap()).unwrap();
            assert_eq!(namespaces[0].0.to_string(), "4t2".to_string());
        }
    }

    mod handle_account_address_query {
        use super::*;

        #[coverage_helper::test]
        fn not_registered_should_be_unknown() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let not_registered = AccountId::new(15, AccountTrace::Local)?;
            let res = query_helper(
                &deps,
                QueryMsg::Accounts {
                    account_ids: vec![not_registered.clone()],
                },
            );

            assert_eq!(
                res,
                Err(RegistryError::Std(StdError::generic_err(
                    RegistryError::UnknownAccountId { id: not_registered }.to_string(),
                )))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn registered_should_return_account() -> RegistryTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_account_querier(deps.api).build();
            mock_init_with_account(&mut deps)?;

            let res = query_helper(
                &deps,
                QueryMsg::Accounts {
                    account_ids: vec![TEST_ACCOUNT_ID],
                },
            );

            let AccountsResponse { accounts } = from_json(res.unwrap()).unwrap();
            assert_eq!(accounts, vec![test_account(deps.api)]);

            Ok(())
        }
    }
}
