use crate::contract::VCResult;
use crate::error::VCError;
use abstract_core::{
    objects::module::ModuleStatus,
    version_control::{state::PENDING_MODULES, ModuleConfiguration, NamespaceResponse},
};
use abstract_sdk::core::{
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
        namespace::Namespace,
        AccountId,
    },
    version_control::{
        namespaces_info,
        state::{ACCOUNT_ADDRESSES, REGISTERED_MODULES, YANKED_MODULES},
        AccountBaseResponse, ModuleFilter, ModuleResponse, ModulesListResponse, ModulesResponse,
        NamespaceListResponse,
    },
};
use cosmwasm_std::{Deps, Order, StdError, StdResult};
use cw_storage_plus::{Bound, Map};

const DEFAULT_LIMIT: u8 = 10;
const MAX_LIMIT: u8 = 20;

pub fn handle_account_address_query(
    deps: Deps,
    account_id: AccountId,
) -> StdResult<AccountBaseResponse> {
    let account_address = ACCOUNT_ADDRESSES.load(deps.storage, account_id);
    match account_address {
        Err(_) => Err(StdError::generic_err(
            VCError::UnknownAccountId { id: account_id }.to_string(),
        )),
        Ok(base) => Ok(AccountBaseResponse { account_base: base }),
    }
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
                .ok_or_else(|| StdError::GenericErr {
                    msg: VCError::ModuleNotFound(module.clone()).to_string(),
                })?
                .clone();
            module.version = ModuleVersion::Version(latest_version);
            Ok(id)
        };

        match maybe_module_ref {
            Err(_) => Err(StdError::generic_err(
                VCError::ModuleNotFound(module).to_string(),
            )),
            Ok(mod_ref) => {
                modules_response.modules.push(ModuleResponse {
                    module: Module {
                        info: module.clone(),
                        reference: mod_ref,
                    },
                    config: ModuleConfiguration::from_storage(deps.storage, &module),
                });
                Ok(())
            }
        }?;
    }

    Ok(modules_response)
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
        Some(ModuleStatus::REGISTERED) => &REGISTERED_MODULES,
        Some(ModuleStatus::PENDING) => &PENDING_MODULES,
        Some(ModuleStatus::YANKED) => &YANKED_MODULES,
        None => &REGISTERED_MODULES,
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
                .collect::<StdResult<Vec<_>>>()?
                .into_iter(),
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
                config: ModuleConfiguration::from_storage(deps.storage, &module_info),
            })
        })
        .collect::<Result<Vec<_>, StdError>>()?;

    Ok(ModulesListResponse { modules })
}

pub fn handle_namespaces_query(
    deps: Deps,
    accounts: Vec<AccountId>,
) -> StdResult<NamespaceListResponse> {
    let mut namespaces_response = NamespaceListResponse { namespaces: vec![] };
    for account_id in accounts {
        namespaces_response.namespaces.extend(
            namespaces_info()
                .idx
                .account_id
                .prefix(account_id)
                .range(deps.storage, None, None, Order::Ascending)
                .collect::<StdResult<Vec<_>>>()?
                .into_iter(),
        );
    }

    Ok(namespaces_response)
}

pub fn handle_namespace_query(deps: Deps, namespace: Namespace) -> StdResult<NamespaceResponse> {
    let account_id = namespaces_info().load(deps.storage, &namespace)?;
    let account_base = ACCOUNT_ADDRESSES.load(deps.storage, account_id)?;

    Ok(NamespaceResponse {
        account_id,
        account_base,
    })
}

pub fn handle_namespace_list_query(
    deps: Deps,
    start_after: Option<Namespace>,
    limit: Option<u8>,
) -> StdResult<NamespaceListResponse> {
    let start_bound = start_after.as_ref().map(Bound::exclusive);
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let namespaces = namespaces_info()
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
    mod_lib: &Map<&ModuleInfo, ModuleReference>,
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
    use abstract_testing::prelude::{
        test_account_base, TEST_ACCOUNT_FACTORY, TEST_ACCOUNT_ID, TEST_MANAGER,
        TEST_MODULE_FACTORY, TEST_VERSION_CONTROL,
    };
    use abstract_testing::{MockQuerierBuilder, MockQuerierOwnership};
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{to_binary, Addr, Binary, DepsMut, StdError, Uint64};

    use abstract_core::{manager, version_control::*};

    use crate::contract;
    use crate::contract::VCResult;
    use speculoos::prelude::*;

    use super::*;

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_ADMIN: &str = "testadmin";

    const TEST_OTHER: &str = "testother";
    const TEST_OTHER_ACCOUNT_ID: u32 = 2;
    const TEST_OTHER_PROXY_ADDR: &str = "proxy1";
    const TEST_OTHER_MANAGER_ADDR: &str = "manager1";

    pub fn mock_manager_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default()
            .with_smart_handler(TEST_MANAGER, |msg| {
                match from_binary(msg).unwrap() {
                    manager::QueryMsg::Config {} => {
                        let resp = manager::ConfigResponse {
                            version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
                            module_factory_address: Addr::unchecked(TEST_MODULE_FACTORY),
                            account_id: Uint64::from(TEST_ACCOUNT_ID), // mock value, not used
                            is_suspended: false,
                        };
                        Ok(to_binary(&resp).unwrap())
                    }
                    _ => panic!("unexpected message"),
                }
            })
            .with_smart_handler(TEST_OTHER_MANAGER_ADDR, |msg| {
                match from_binary(msg).unwrap() {
                    manager::QueryMsg::Config {} => {
                        let resp = manager::ConfigResponse {
                            version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
                            module_factory_address: Addr::unchecked(TEST_MODULE_FACTORY),
                            account_id: Uint64::from(TEST_OTHER_ACCOUNT_ID), // mock value, not used
                            is_suspended: false,
                        };
                        Ok(to_binary(&resp).unwrap())
                    }
                    _ => panic!("unexpected message"),
                }
            })
            .with_owner(TEST_MANAGER, Some(TEST_ADMIN))
            .with_owner(TEST_OTHER_MANAGER_ADDR, Some(TEST_OTHER))
    }

    fn mock_init(mut deps: DepsMut) -> VersionControlTestResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(
            deps.branch(),
            mock_env(),
            info,
            InstantiateMsg {
                allow_direct_module_registration_and_updates: Some(true),
                namespace_registration_fee: None,
            },
        )?;
        execute_as_admin(
            deps.branch(),
            ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.to_string(),
            },
        )?;

        Ok(())
    }

    /// Initialize the version_control with admin as creator and test account
    fn mock_init_with_account(mut deps: DepsMut) -> VCResult {
        mock_init(deps.branch())?;
        execute_as(
            deps.branch(),
            TEST_ACCOUNT_FACTORY,
            ExecuteMsg::AddAccount {
                account_id: TEST_ACCOUNT_ID,
                account_base: test_account_base(),
            },
        )?;
        execute_as(
            deps.branch(),
            TEST_ACCOUNT_FACTORY,
            ExecuteMsg::AddAccount {
                account_id: TEST_OTHER_ACCOUNT_ID,
                account_base: AccountBase {
                    manager: Addr::unchecked(TEST_OTHER_MANAGER_ADDR),
                    proxy: Addr::unchecked(TEST_OTHER_PROXY_ADDR),
                },
            },
        )
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(TEST_ADMIN, &[]), msg)
    }

    fn query_helper(deps: Deps, msg: QueryMsg) -> VCResult<Binary> {
        contract::query(deps, mock_env(), msg)
    }

    mod module {
        use super::*;
        use abstract_core::objects::module::ModuleVersion::Latest;

        use cosmwasm_std::from_binary;

        fn add_namespace(deps: DepsMut, namespace: &str) {
            let msg = ExecuteMsg::ClaimNamespace {
                account_id: TEST_ACCOUNT_ID,
                namespace: namespace.to_string(),
            };

            let res = execute_as_admin(deps, msg);
            assert_that!(&res).is_ok();
        }

        fn add_module(deps: DepsMut, new_module_info: ModuleInfo) {
            let add_msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module_info, ModuleReference::App(0))],
            };

            let res = execute_as_admin(deps, add_msg);
            assert_that!(&res).is_ok();
        }

        #[test]
        fn get_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo {
                name, namespace, ..
            } = new_module_info.clone();

            add_namespace(deps.as_mut(), "test");
            add_module(deps.as_mut(), new_module_info.clone());

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace,
                    name,
                    version: Latest {},
                }],
            };

            let ModulesResponse { mut modules } =
                from_binary(&query_helper(deps.as_ref(), query_msg)?)?;
            assert_that!(modules.swap_remove(0).module.info).is_equal_to(&new_module_info);
            Ok(())
        }

        #[test]
        fn none_when_no_matching_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo {
                name, namespace, ..
            } = new_module_info.clone();

            add_namespace(deps.as_mut(), "test");
            add_module(deps.as_mut(), new_module_info);

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace,
                    name,
                    version: ModuleVersion::Version("024209.902.902".to_string()),
                }],
            };

            let res = query_helper(deps.as_ref(), query_msg);
            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, VCError::Std(StdError::GenericErr { .. })));
            Ok(())
        }

        #[test]
        fn get_latest_when_multiple_registered() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut())?;

            add_namespace(deps.as_mut(), "test");

            let module_id = "test:module";
            let oldest_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("0.1.2".into())).unwrap();

            add_module(deps.as_mut(), oldest_version);
            let newest_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("100.1.2".into())).unwrap();
            add_module(deps.as_mut(), newest_version.clone());

            let another_version =
                ModuleInfo::from_id(module_id, ModuleVersion::Version("1.1.2".into())).unwrap();
            add_module(deps.as_mut(), another_version);

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace: Namespace::new("test")?,
                    name: "module".to_string(),
                    version: Latest {},
                }],
            };

            let ModulesResponse { mut modules } =
                from_binary(&query_helper(deps.as_ref(), query_msg)?)?;
            assert_that!(modules.swap_remove(0).module.info).is_equal_to(&newest_version);
            Ok(())
        }
    }

    use cosmwasm_std::from_binary;

    /// Add namespaces
    fn add_namespaces(mut deps: DepsMut, acc_and_namespace: Vec<(u32, &str)>, sender: &str) {
        for (account_id, namespace) in acc_and_namespace {
            let msg = ExecuteMsg::ClaimNamespace {
                account_id,
                namespace: namespace.to_string(),
            };

            let res = execute_as(deps.branch(), sender, msg);
            assert_that!(&res).is_ok();
        }
    }

    /// Add the provided modules to the version control
    fn propose_modules(deps: DepsMut, new_module_infos: Vec<ModuleInfo>, sender: &str) {
        let modules = new_module_infos
            .into_iter()
            .map(|info| (info, ModuleReference::App(0)))
            .collect();
        let add_msg = ExecuteMsg::ProposeModules { modules };
        let res = execute_as(deps, sender, add_msg);
        assert_that!(&res).is_ok();
    }

    /// Yank the provided module in the version control
    fn yank_module(deps: DepsMut, module_info: ModuleInfo) {
        let yank_msg = ExecuteMsg::YankModule {
            module: module_info,
        };
        let res = execute_as_admin(deps, yank_msg);
        assert_that!(&res).is_ok();
    }

    /// Init verison control with some test modules.
    fn init_with_mods(mut deps: DepsMut) {
        mock_init_with_account(deps.branch()).unwrap();

        add_namespaces(
            deps.branch(),
            vec![(TEST_ACCOUNT_ID, "cw-plus")],
            TEST_ADMIN,
        );
        add_namespaces(deps.branch(), vec![(2, "4t2")], TEST_OTHER);

        let cw_mods = vec![
            ModuleInfo::from_id("cw-plus:module1", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("cw-plus:module2", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("cw-plus:module3", ModuleVersion::Version("0.1.2".into())).unwrap(),
        ];
        propose_modules(deps.branch(), cw_mods, TEST_ADMIN);

        let fortytwo_mods = vec![
            ModuleInfo::from_id("4t2:module1", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("4t2:module2", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("4t2:module3", ModuleVersion::Version("0.1.2".into())).unwrap(),
        ];
        propose_modules(deps, fortytwo_mods, TEST_OTHER);
    }

    mod modules {
        use super::*;

        #[test]
        fn get_cw_plus_modules() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

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

            let ModulesResponse { modules } =
                from_binary(&query_helper(deps.as_ref(), query_msg)?)?;
            assert_that!(modules).has_length(3);
            for module in modules {
                assert_that!(module.module.info.namespace).is_equal_to(namespace.clone());
                assert_that!(module.module.info.version)
                    .is_equal_to(&ModuleVersion::Version("0.1.2".into()));
            }
            Ok(())
        }

        #[test]
        fn get_modules_not_found() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace: Namespace::new("not")?,
                    name: "found".to_string(),
                    version: ModuleVersion::Latest {},
                }],
            };

            let res = query_helper(deps.as_ref(), query_msg);
            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, VCError::Std(StdError::GenericErr { .. })));
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

        #[test]
        fn filter_by_namespace_existing() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());
            let filtered_namespace = "cw-plus".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                ..Default::default()
            };
            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(3);

                for entry in modules {
                    assert_that!(entry.module.info.namespace)
                        .is_equal_to(Namespace::unchecked(filtered_namespace.clone()));
                }

                res
            });
        }

        #[test]
        fn filter_default_returns_only_non_yanked() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let cw_mods = vec![
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            ];
            propose_modules(deps.as_mut(), cw_mods, TEST_ADMIN);
            yank_module(
                deps.as_mut(),
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );
            yank_module(
                deps.as_mut(),
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );

            let list_msg = QueryMsg::ModuleList {
                filter: None,
                start_after: None,
                limit: None,
            };

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(6);

                let yanked_module_names = ["module4".to_string(), "module5".to_string()];
                for entry in modules {
                    if entry.module.info.namespace == Namespace::unchecked("cw-plus") {
                        assert!(!yanked_module_names
                            .iter()
                            .any(|e| e == &entry.module.info.name));
                    }
                }

                res
            });
        }

        #[test]
        fn filter_yanked_by_namespace_existing() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let cw_mods = vec![
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            ];
            propose_modules(deps.as_mut(), cw_mods, TEST_ADMIN);
            yank_module(
                deps.as_mut(),
                ModuleInfo::from_id("cw-plus:module4", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );
            yank_module(
                deps.as_mut(),
                ModuleInfo::from_id("cw-plus:module5", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            );

            let filtered_namespace = "cw-plus".to_string();

            let filter = ModuleFilter {
                status: Some(ModuleStatus::YANKED),
                namespace: Some(filtered_namespace.clone()),
                ..Default::default()
            };
            let list_msg = QueryMsg::ModuleList {
                filter: Some(filter),
                start_after: None,
                limit: None,
            };

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(2);

                for entry in modules {
                    assert_that!(entry.module.info.namespace)
                        .is_equal_to(Namespace::unchecked(filtered_namespace.clone()));
                }

                res
            });
        }

        #[test]
        fn filter_by_namespace_non_existing() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut()).unwrap();
            add_namespaces(
                deps.as_mut(),
                vec![(TEST_ACCOUNT_ID, "cw-plus")],
                TEST_ADMIN,
            );
            let cw_mods = vec![ModuleInfo::from_id(
                "cw-plus:module1",
                ModuleVersion::Version("0.1.2".into()),
            )
            .unwrap()];
            propose_modules(deps.as_mut(), cw_mods, TEST_ADMIN);

            let filtered_namespace = "cw-plus".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(1);

                res
            });
        }

        #[test]
        fn filter_by_namespace_and_name() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let filtered_namespace = "cw-plus".to_string();
            let filtered_name = "module2".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                name: Some(filtered_name.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(1);

                let module = modules[0].clone();
                assert_that!(module.module.info.namespace)
                    .is_equal_to(Namespace::unchecked(filtered_namespace.clone()));
                assert_that!(module.module.info.name).is_equal_to(filtered_name.clone());
                res
            });
        }

        #[test]
        fn filter_by_namespace_and_name_with_multiple_versions() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let filtered_namespace = "cw-plus".to_string();
            let filtered_name = "module2".to_string();

            propose_modules(
                deps.as_mut(),
                vec![ModuleInfo::from_id(
                    format!("{filtered_namespace}:{filtered_name}").as_str(),
                    ModuleVersion::Version("0.1.3".into()),
                )
                .unwrap()],
                TEST_ADMIN,
            );

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                name: Some(filtered_name.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(2);

                for module in modules {
                    assert_that!(module.module.info.namespace)
                        .is_equal_to(Namespace::unchecked(filtered_namespace.clone()));
                    assert_that!(module.module.info.name).is_equal_to(filtered_name.clone());
                }
                res
            });
        }

        #[test]
        fn filter_by_only_version_many() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let filtered_version = "0.1.2".to_string();

            let filter = ModuleFilter {
                version: Some(filtered_version.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(6);

                for module in modules {
                    assert_that!(module.module.info.version.to_string())
                        .is_equal_to(filtered_version.clone());
                }
                res
            });
        }

        #[test]
        fn filter_by_only_version_none() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let filtered_version = "5555".to_string();

            let filter = ModuleFilter {
                version: Some(filtered_version),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).is_empty();

                res
            });
        }

        #[test]
        fn filter_by_name_and_version() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let filtered_name = "module2".to_string();
            let filtered_version = "0.1.2".to_string();

            let filter = ModuleFilter {
                name: Some(filtered_name.clone()),
                version: Some(filtered_version.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                // We expect two because both cw-plus and snth have a module2 with version 0.1.2
                assert_that!(modules).has_length(2);

                for module in modules {
                    assert_that!(module.module.info.name).is_equal_to(filtered_name.clone());
                    assert_that!(module.module.info.version.to_string())
                        .is_equal_to(filtered_version.clone());
                }
                res
            });
        }

        #[test]
        fn filter_by_namespace_and_version() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            let filtered_namespace = "cw-plus".to_string();
            let filtered_version = "0.1.2".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace.clone()),
                version: Some(filtered_version.clone()),
                ..Default::default()
            };

            let list_msg = filtered_list_msg(filter);

            let res = query_helper(deps.as_ref(), list_msg);

            assert_that!(res).is_ok().map(|res| {
                let ModulesListResponse { modules } = from_binary(res).unwrap();
                assert_that!(modules).has_length(3);

                for module in modules {
                    assert_that!(module.module.info.namespace)
                        .is_equal_to(Namespace::unchecked(filtered_namespace.clone()));
                    assert_that!(module.module.info.version.to_string())
                        .is_equal_to(filtered_version.clone());
                }

                res
            });
        }
    }

    mod query_namespaces {
        use super::*;

        #[test]
        fn namespaces() {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            init_with_mods(deps.as_mut());

            // get for test other account
            let res = query_helper(
                deps.as_ref(),
                QueryMsg::Namespaces {
                    accounts: vec![TEST_OTHER_ACCOUNT_ID],
                },
            );
            assert_that!(res).is_ok().map(|res| {
                let NamespacesResponse { namespaces } = from_binary(res).unwrap();
                assert_that!(namespaces[0].0.to_string()).is_equal_to("4t2".to_string());
                res
            });
        }
    }

    mod handle_account_address_query {
        use super::*;
        use abstract_testing::prelude::test_account_base;

        #[test]
        fn not_registered_should_be_unknown() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let not_registered = 15;
            let res = query_helper(
                deps.as_ref(),
                QueryMsg::AccountBase {
                    account_id: not_registered,
                },
            );

            // let res2 = from_binary(&res.unwrap())?;

            assert_that!(res)
                .is_err()
                .is_equal_to(VCError::Std(StdError::generic_err(
                    VCError::UnknownAccountId { id: not_registered }.to_string(),
                )));

            Ok(())
        }

        #[test]
        fn registered_should_return_account_base() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init_with_account(deps.as_mut())?;

            let res = query_helper(
                deps.as_ref(),
                QueryMsg::AccountBase {
                    account_id: TEST_ACCOUNT_ID,
                },
            );

            assert_that!(res).is_ok().map(|res| {
                let AccountBaseResponse { account_base } = from_binary(res).unwrap();
                assert_that!(account_base).is_equal_to(test_account_base());
                res
            });

            Ok(())
        }
    }
}
