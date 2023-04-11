use crate::error::VCError;
use abstract_core::objects::AccountId;
use abstract_core::version_control::ModuleFilter;
use abstract_sdk::core::{
    objects::{
        module::{Module, ModuleInfo, ModuleVersion},
        module_reference::ModuleReference,
    },
    version_control::{
        state::ACCOUNT_ADDRESSES, state::MODULE_LIBRARY, AccountBaseResponse, ModulesListResponse,
        ModulesResponse,
    },
};
use cosmwasm_std::{to_binary, Binary, Deps, Order, StdError, StdResult};
use cw_storage_plus::Bound;

const DEFAULT_LIMIT: u8 = 10;
const MAX_LIMIT: u8 = 20;

pub fn handle_account_address_query(deps: Deps, account_id: AccountId) -> StdResult<Binary> {
    let account_address = ACCOUNT_ADDRESSES.load(deps.storage, account_id);
    match account_address {
        Err(_) => Err(StdError::generic_err(
            VCError::MissingAccountId { id: account_id }.to_string(),
        )),
        Ok(base) => to_binary(&AccountBaseResponse { account_base: base }),
    }
}

pub fn handle_modules_query(deps: Deps, modules: Vec<ModuleInfo>) -> StdResult<Binary> {
    let mut modules_response = ModulesResponse { modules: vec![] };
    for mut module in modules {
        let maybe_module_ref = if let ModuleVersion::Version(_) = module.version {
            MODULE_LIBRARY.load(deps.storage, &module)
        } else {
            // get latest
            let versions: StdResult<Vec<(String, ModuleReference)>> = MODULE_LIBRARY
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
                modules_response.modules.push(Module {
                    info: module,
                    reference: mod_ref,
                });
                Ok(())
            }
        }?;
    }

    to_binary(&modules_response)
}

pub fn handle_module_list_query(
    deps: Deps,
    start_after: Option<ModuleInfo>,
    limit: Option<u8>,
    filter: Option<ModuleFilter>,
) -> StdResult<Binary> {
    let limit = limit.unwrap_or(DEFAULT_LIMIT).min(MAX_LIMIT) as usize;

    let mut modules: Vec<(ModuleInfo, ModuleReference)> = vec![];

    let ModuleFilter {
        namespace: ref namespace_filter,
        name: ref name_filter,
        version: version_filter,
    } = filter.unwrap_or_default();

    if let Some(namespace_filter) = namespace_filter {
        modules.extend(filter_modules_by_namespace(
            deps,
            start_after,
            limit,
            namespace_filter,
            name_filter,
        )?);
    } else {
        let start_bound: Option<Bound<&ModuleInfo>> = start_after.as_ref().map(Bound::exclusive);

        // Load all modules
        modules.extend(
            MODULE_LIBRARY
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

    let modules = modules.into_iter().map(Module::from).collect();

    to_binary(&ModulesListResponse { modules })
}

/// Filter the modules with their primary key prefix (namespace)
fn filter_modules_by_namespace(
    deps: Deps,
    start_after: Option<ModuleInfo>,
    limit: usize,
    namespace: &str,
    name: &Option<String>,
) -> StdResult<Vec<(ModuleInfo, ModuleReference)>> {
    let mut modules: Vec<(ModuleInfo, ModuleReference)> = vec![];

    // Filter by name using full prefix
    if let Some(name) = name {
        let start_bound: Option<Bound<String>> =
            start_after.map(|token| Bound::exclusive(token.namespace));

        modules.extend(
            MODULE_LIBRARY
                .prefix((namespace.to_owned(), name.clone()))
                .range(deps.storage, start_bound, None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?
                .into_iter()
                .map(|(version, reference)| {
                    (
                        ModuleInfo {
                            namespace: namespace.to_owned(),
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
            start_after.map(|token| Bound::exclusive((token.namespace, token.name)));

        modules.extend(
            MODULE_LIBRARY
                .sub_prefix(namespace.to_owned())
                .range(deps.storage, start_bound, None, Order::Ascending)
                .take(limit)
                .collect::<StdResult<Vec<_>>>()?
                .into_iter()
                .map(|((name, version), reference)| {
                    (
                        ModuleInfo {
                            namespace: namespace.to_owned(),
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
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{DepsMut, StdError};

    use abstract_core::version_control::*;

    use crate::contract;
    use crate::contract::VCResult;
    use speculoos::prelude::*;

    use super::*;

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_ADMIN: &str = "testadmin";

    /// Initialize the version_control with admin as creator and factory
    fn mock_init(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(deps.branch(), mock_env(), info, InstantiateMsg {})
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(TEST_ADMIN, &[]), msg)
    }

    fn query_helper(deps: Deps, msg: QueryMsg) -> StdResult<Binary> {
        contract::query(deps, mock_env(), msg)
    }

    mod module {
        use super::*;
        use abstract_core::objects::module::ModuleVersion::Latest;

        use cosmwasm_std::from_binary;

        fn add_module(deps: DepsMut, new_module_info: ModuleInfo) {
            let add_msg = ExecuteMsg::AddModules {
                modules: vec![(new_module_info, ModuleReference::App(0))],
            };

            let res = execute_as_admin(deps, add_msg);
            assert_that!(&res).is_ok();
        }

        #[test]
        fn get_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo {
                name, namespace, ..
            } = new_module_info.clone();

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
            assert_that!(modules.swap_remove(0).info).is_equal_to(&new_module_info);
            Ok(())
        }

        #[test]
        fn none_when_no_matching_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;
            let new_module_info =
                ModuleInfo::from_id("test:module", ModuleVersion::Version("0.1.2".into())).unwrap();

            let ModuleInfo {
                name, namespace, ..
            } = new_module_info.clone();

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
                .matches(|e| matches!(e, StdError::GenericErr { .. }));
            Ok(())
        }

        #[test]
        fn get_latest_when_multiple_registered() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

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
                    namespace: "test".to_string(),
                    name: "module".to_string(),
                    version: Latest {},
                }],
            };

            let ModulesResponse { mut modules } =
                from_binary(&query_helper(deps.as_ref(), query_msg)?)?;
            assert_that!(modules.swap_remove(0).info).is_equal_to(&newest_version);
            Ok(())
        }
    }

    use cosmwasm_std::from_binary;

    /// Add the provided modules to the version control
    fn add_modules(deps: DepsMut, new_module_infos: Vec<ModuleInfo>) {
        let modules = new_module_infos
            .into_iter()
            .map(|info| (info, ModuleReference::App(0)))
            .collect();
        let add_msg = ExecuteMsg::AddModules { modules };
        let res = execute_as_admin(deps, add_msg);
        assert_that!(&res).is_ok();
    }

    /// Init verison control with some test modules.
    fn init_with_mods(mut deps: DepsMut) {
        mock_init(deps.branch()).unwrap();
        let cw_mods = vec![
            ModuleInfo::from_id("cw-plus:module1", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("cw-plus:module2", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("cw-plus:module3", ModuleVersion::Version("0.1.2".into())).unwrap(),
        ];
        add_modules(deps.branch(), cw_mods);

        let fortytwo_mods = vec![
            ModuleInfo::from_id("4t2:module1", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("4t2:module2", ModuleVersion::Version("0.1.2".into())).unwrap(),
            ModuleInfo::from_id("4t2:module3", ModuleVersion::Version("0.1.2".into())).unwrap(),
        ];
        add_modules(deps, fortytwo_mods);
    }

    mod modules {
        use super::*;

        #[test]
        fn get_cw_plus_modules() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            init_with_mods(deps.as_mut());

            let namespace = "cw-plus".to_string();

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
                assert_that!(module.info.namespace).is_equal_to(namespace.clone());
                assert_that!(module.info.version)
                    .is_equal_to(&ModuleVersion::Version("0.1.2".into()));
            }
            Ok(())
        }

        #[test]
        fn get_modules_not_found() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            init_with_mods(deps.as_mut());

            let query_msg = QueryMsg::Modules {
                infos: vec![ModuleInfo {
                    namespace: "not".to_string(),
                    name: "found".to_string(),
                    version: ModuleVersion::Latest {},
                }],
            };

            let res = query_helper(deps.as_ref(), query_msg);
            assert_that!(res)
                .is_err()
                .matches(|e| matches!(e, StdError::GenericErr { .. }));
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
                    assert_that!(entry.info.namespace).is_equal_to(filtered_namespace.clone());
                }

                res
            });
        }

        #[test]
        fn filter_by_namespace_non_existing() {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut()).unwrap();
            let cw_mods = vec![
                ModuleInfo::from_id("cw-plus:module1", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
                ModuleInfo::from_id("aoeu:module2", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
                ModuleInfo::from_id("snth:module3", ModuleVersion::Version("0.1.2".into()))
                    .unwrap(),
            ];
            add_modules(deps.as_mut(), cw_mods);

            let filtered_namespace = "dne".to_string();

            let filter = ModuleFilter {
                namespace: Some(filtered_namespace),
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
        fn filter_by_namespace_and_name() {
            let mut deps = mock_dependencies();

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
                assert_that!(module.info.namespace).is_equal_to(filtered_namespace.clone());
                assert_that!(module.info.name).is_equal_to(filtered_name.clone());
                res
            });
        }

        #[test]
        fn filter_by_namespace_and_name_with_multiple_versions() {
            let mut deps = mock_dependencies();

            init_with_mods(deps.as_mut());

            let filtered_namespace = "cw-plus".to_string();
            let filtered_name = "module2".to_string();

            add_modules(
                deps.as_mut(),
                vec![ModuleInfo::from_id(
                    format!("{filtered_namespace}:{filtered_name}").as_str(),
                    ModuleVersion::Version("0.1.3".into()),
                )
                .unwrap()],
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
                    assert_that!(module.info.namespace).is_equal_to(filtered_namespace.clone());
                    assert_that!(module.info.name).is_equal_to(filtered_name.clone());
                }
                res
            });
        }

        #[test]
        fn filter_by_only_version_many() {
            let mut deps = mock_dependencies();

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
                    assert_that!(module.info.version.to_string())
                        .is_equal_to(filtered_version.clone());
                }
                res
            });
        }

        #[test]
        fn filter_by_only_version_none() {
            let mut deps = mock_dependencies();

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
                    assert_that!(module.info.name).is_equal_to(filtered_name.clone());
                    assert_that!(module.info.version.to_string())
                        .is_equal_to(filtered_version.clone());
                }
                res
            });
        }

        #[test]
        fn filter_by_namespace_and_version() {
            let mut deps = mock_dependencies();

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
                    assert_that!(module.info.namespace).is_equal_to(filtered_namespace.clone());
                    assert_that!(module.info.version.to_string())
                        .is_equal_to(filtered_version.clone());
                }

                res
            });
        }
    }
}
