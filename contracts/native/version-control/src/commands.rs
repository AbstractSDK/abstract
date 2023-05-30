use abstract_core::objects::module::{self, Module};
use cosmwasm_std::{
    ensure, Addr, Attribute, Deps, DepsMut, MessageInfo, Order, QuerierWrapper, Response,
    StdResult, Storage,
};

use abstract_sdk::{
    core::{
        objects::{
            common_namespace::OWNERSHIP_STORAGE_KEY,
            module::{ModuleInfo, ModuleVersion},
            module_reference::ModuleReference,
            namespace::Namespace,
            AccountId,
        },
        version_control::{namespaces_info, state::*, AccountBase, Config},
    },
    cw_helpers::wasm_raw_query,
};

use crate::contract::{VCResult, VcResponse, ABSTRACT_NAMESPACE};
use crate::error::VCError;

/// Add new Account to version control contract
/// Only Factory can add Account
pub fn add_account(
    deps: DepsMut,
    msg_info: MessageInfo,
    account_id: AccountId,
    account_base: AccountBase,
) -> VCResult {
    // Only Factory can add new Account
    FACTORY.assert_admin(deps.as_ref(), &msg_info.sender)?;
    ACCOUNT_ADDRESSES.save(deps.storage, account_id, &account_base)?;

    Ok(VcResponse::new(
        "add_account",
        vec![
            ("account_id", account_id.to_string().as_str()),
            ("manager", account_base.manager.as_ref()),
            ("proxy", account_base.proxy.as_ref()),
        ],
    ))
}

/// Here we can add logic to allow subscribers to claim a namespace and upload contracts to that namespace
pub fn propose_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    modules: Vec<(ModuleInfo, ModuleReference)>,
) -> VCResult {
    let config = CONFIG.load(deps.storage)?;

    for (module, mod_ref) in modules {
        if PENDING_MODULES.has(deps.storage, &module)
            || REGISTERED_MODULES.has(deps.storage, &module)
            || YANKED_MODULES.has(deps.storage, &module)
        {
            return Err(VCError::NotUpdateableModule(module));
        }

        module.validate()?;

        mod_ref.validate(deps.as_ref())?;

        // version must be set in order to add the new version
        module.assert_version_variant()?;

        if module.namespace == Namespace::unchecked(ABSTRACT_NAMESPACE) {
            // Only Admin can update abstract contracts
            cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;
        } else {
            // Only owner can add modules
            validate_account_owner(deps.as_ref(), &module.namespace, &msg_info.sender)?;
        }

        // verify contract admin is None if module is Adapter
        if let ModuleReference::Adapter(ref addr) = mod_ref {
            if deps.querier.query_wasm_contract_info(addr)?.admin.is_some() {
                return Err(VCError::AdminMustBeNone);
            }
        }

        // assert that its data is equal to what it wants to be registered under.
        module::assert_module_data_validity(
            &deps.querier,
            &Module {
                info: module.clone(),
                reference: mod_ref.clone(),
            },
            None,
        )?;

        if config.is_testnet {
            REGISTERED_MODULES.save(deps.storage, &module, &mod_ref)?;
        } else {
            PENDING_MODULES.save(deps.storage, &module, &mod_ref)?;
        }
    }

    Ok(VcResponse::action("propose_modules"))
}

/// Approve and reject modules
pub fn approve_or_reject_modules(
    deps: DepsMut,
    msg_info: MessageInfo,
    approves: Vec<ModuleInfo>,
    rejects: Vec<ModuleInfo>,
) -> VCResult {
    // Only Admin can approve or rejects a module
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    let mut attributes = vec![];
    if !approves.is_empty() {
        attributes.push(approve_modules(deps.storage, approves)?);
    }
    if !rejects.is_empty() {
        attributes.push(reject_modules(deps.storage, rejects)?);
    }
    if attributes.is_empty() {
        return Err(VCError::NoAction);
    }

    Ok(VcResponse::new("approve_or_reject_modules", attributes))
}

/// Admin approve modules
fn approve_modules(storage: &mut dyn Storage, approves: Vec<ModuleInfo>) -> VCResult<Attribute> {
    for module in &approves {
        let mod_ref = PENDING_MODULES
            .may_load(storage, module)?
            .ok_or_else(|| VCError::ModuleNotFound(module.clone()))?;
        // Register the module
        REGISTERED_MODULES.save(storage, module, &mod_ref)?;
        // Remove from pending
        PENDING_MODULES.remove(storage, module);
    }

    let approves: Vec<_> = approves.into_iter().map(|m| m.to_string()).collect();
    Ok(("approves", approves.join(",")).into())
}

/// Admin reject modules
fn reject_modules(storage: &mut dyn Storage, rejects: Vec<ModuleInfo>) -> VCResult<Attribute> {
    for module in &rejects {
        if !PENDING_MODULES.has(storage, module) {
            return Err(VCError::ModuleNotFound(module.clone()));
        }
        PENDING_MODULES.remove(storage, module);
    }

    let rejects: Vec<_> = rejects.into_iter().map(|m| m.to_string()).collect();
    Ok(("rejects", rejects.join(",")).into())
}

/// Remove a module from the Version Control registry.
pub fn remove_module(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // Only the Version Control Admin can remove modules
    cw_ownable::assert_owner(deps.storage, &msg_info.sender)?;

    // Only specific versions may be removed
    module.assert_version_variant()?;

    ensure!(
        REGISTERED_MODULES.has(deps.storage, &module) || YANKED_MODULES.has(deps.storage, &module),
        VCError::ModuleNotFound(module)
    );

    REGISTERED_MODULES.remove(deps.storage, &module);
    YANKED_MODULES.remove(deps.storage, &module);

    Ok(VcResponse::new(
        "remove_module",
        vec![("module", &module.to_string())],
    ))
}

/// Yank a module, preventing it from being used.
pub fn yank_module(deps: DepsMut, msg_info: MessageInfo, module: ModuleInfo) -> VCResult {
    // validate the caller is the owner of the namespace
    validate_account_owner(deps.as_ref(), &module.namespace, &msg_info.sender)?;

    // Only specific versions may be yanked
    module.assert_version_variant()?;
    let mod_ref = REGISTERED_MODULES
        .may_load(deps.storage, &module)?
        .ok_or_else(|| VCError::ModuleNotFound(module.clone()))?;

    YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
    REGISTERED_MODULES.remove(deps.storage, &module);

    Ok(VcResponse::new(
        "yank_module",
        vec![("module", &module.to_string())],
    ))
}

/// Claim namespaces
/// Only the Account Owner can do this
pub fn claim_namespaces(
    deps: DepsMut,
    msg_info: MessageInfo,
    account_id: AccountId,
    namespaces_to_claim: Vec<String>,
) -> VCResult {
    // verify account owner
    let account_base = ACCOUNT_ADDRESSES.load(deps.storage, account_id)?;
    let account_owner = query_account_owner(&deps.querier, &account_base.manager, account_id)?;
    if msg_info.sender != account_owner {
        return Err(VCError::AccountOwnerMismatch {
            sender: msg_info.sender,
            owner: account_owner,
        });
    }

    let Config {
        namespace_limit: namespaces_limit,
        ..
    } = CONFIG.load(deps.storage)?;
    let limit = namespaces_limit as usize;
    let existing_namespace_count = namespaces_info()
        .idx
        .account_id
        .prefix(account_id)
        .range(deps.storage, None, None, Order::Ascending)
        .count();
    if existing_namespace_count + namespaces_to_claim.len() > limit {
        return Err(VCError::ExceedsNamespaceLimit {
            limit,
            current: existing_namespace_count,
        });
    }

    for namespace in namespaces_to_claim.iter() {
        let namespace = Namespace::try_from(namespace)?;
        if let Some(id) = namespaces_info().may_load(deps.storage, &namespace)? {
            return Err(VCError::NamespaceOccupied {
                namespace: namespace.to_string(),
                id,
            });
        }
        namespaces_info().save(deps.storage, &namespace, &account_id)?;
    }

    Ok(VcResponse::new(
        "claim_namespaces",
        vec![
            ("account_id", &account_id.to_string()),
            ("namespaces", &namespaces_to_claim.join(",")),
        ],
    ))
}

/// Remove namespaces
/// Only admin or the account owner can do this
pub fn remove_namespaces(
    deps: DepsMut,
    msg_info: MessageInfo,
    namespaces: Vec<String>,
) -> VCResult {
    let is_admin = cw_ownable::is_owner(deps.storage, &msg_info.sender)?;

    let mut logs = vec![];
    for namespace in namespaces.iter() {
        let namespace = Namespace::try_from(namespace)?;
        if !namespaces_info().has(deps.storage, &namespace) {
            return Err(VCError::UnknownNamespace { namespace });
        }
        if !is_admin {
            validate_account_owner(deps.as_ref(), &namespace, &msg_info.sender)?;
        }

        for ((name, version), mod_ref) in REGISTERED_MODULES
            .sub_prefix(namespace.clone())
            .range(deps.storage, None, None, Order::Ascending)
            .collect::<StdResult<Vec<_>>>()?
            .into_iter()
        {
            let module = ModuleInfo {
                namespace: namespace.clone(),
                name,
                version: ModuleVersion::Version(version),
            };
            REGISTERED_MODULES.remove(deps.storage, &module);
            YANKED_MODULES.save(deps.storage, &module, &mod_ref)?;
        }

        logs.push(format!(
            "({}, {})",
            namespace,
            namespaces_info().load(deps.storage, &namespace)?
        ));
        namespaces_info().remove(deps.storage, &namespace)?;
    }

    Ok(VcResponse::new(
        "remove_namespaces",
        vec![("namespaces", &logs.join(","))],
    ))
}

pub fn update_namespace_limit(deps: DepsMut, info: MessageInfo, new_limit: u32) -> VCResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;
    let mut config = CONFIG.load(deps.storage)?;
    let previous_limit = config.namespace_limit;
    ensure!(
        new_limit > previous_limit,
        VCError::DecreaseNamespaceLimit {
            limit: new_limit,
            current: previous_limit,
        }
    );
    config.namespace_limit = new_limit;
    CONFIG.save(deps.storage, &config)?;

    Ok(VcResponse::new(
        "update_namespace_limit",
        vec![
            ("previous_limit", previous_limit.to_string()),
            ("limit", new_limit.to_string()),
        ],
    ))
}

pub fn query_account_owner(
    querier: &QuerierWrapper,
    manager_addr: &Addr,
    account_id: AccountId,
) -> VCResult<Addr> {
    let req = wasm_raw_query(manager_addr, OWNERSHIP_STORAGE_KEY)?;
    let cw_ownable::Ownership { owner, .. } = querier.query(&req)?;

    owner.ok_or(VCError::NoAccountOwner { account_id })
}

pub fn validate_account_owner(
    deps: Deps,
    namespace: &Namespace,
    sender: &Addr,
) -> Result<(), VCError> {
    let sender = sender.clone();
    let account_id = namespaces_info()
        .may_load(deps.storage, &namespace.clone())?
        .ok_or_else(|| VCError::UnknownNamespace {
            namespace: namespace.to_owned(),
        })?;
    let account_base = ACCOUNT_ADDRESSES.load(deps.storage, account_id)?;
    let account_owner = query_account_owner(&deps.querier, &account_base.manager, account_id)?;
    if sender != account_owner {
        return Err(VCError::AccountOwnerMismatch {
            sender,
            owner: account_owner,
        });
    }
    Ok(())
}

pub fn set_factory(deps: DepsMut, info: MessageInfo, new_admin: String) -> VCResult {
    cw_ownable::assert_owner(deps.storage, &info.sender)?;

    let new_factory_addr = deps.api.addr_validate(&new_admin)?;
    FACTORY.set(deps, Some(new_factory_addr))?;
    Ok(Response::new().add_attribute("set_factory", new_admin))
}

#[cfg(test)]
mod test {
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info};
    use cosmwasm_std::{from_binary, to_binary, Addr, Uint64};
    use cw_controllers::AdminError;
    use cw_ownable::OwnershipError;
    use speculoos::prelude::*;

    use abstract_core::manager::ConfigResponse as ManagerConfigResponse;
    use abstract_core::version_control::*;
    use abstract_testing::prelude::TEST_MODULE_ID;
    use abstract_testing::prelude::{
        TEST_ACCOUNT_FACTORY, TEST_ACCOUNT_ID, TEST_ADMIN, TEST_MODULE_FACTORY, TEST_NAMESPACE,
        TEST_VERSION, TEST_VERSION_CONTROL,
    };
    use abstract_testing::MockQuerierBuilder;

    use crate::contract;

    use super::*;
    use crate::testing::*;
    use abstract_core::manager::QueryMsg as ManagerQueryMsg;
    use abstract_testing::prelude::*;
    use abstract_testing::MockQuerierOwnership;

    type VersionControlTestResult = Result<(), VCError>;

    const TEST_OTHER: &str = "test-other";

    pub fn mock_manager_querier() -> MockQuerierBuilder {
        MockQuerierBuilder::default()
            .with_smart_handler(TEST_MANAGER, |msg| {
                match from_binary(msg).unwrap() {
                    ManagerQueryMsg::Config {} => {
                        let resp = ManagerConfigResponse {
                            version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
                            module_factory_address: Addr::unchecked(TEST_MODULE_FACTORY),
                            account_id: Uint64::from(TEST_ACCOUNT_ID), // mock value, not used
                            is_suspended: false,
                        };
                        Ok(to_binary(&resp).unwrap())
                    }
                    ManagerQueryMsg::Ownership {} => {
                        let resp = cw_ownable::Ownership {
                            owner: Some(Addr::unchecked(TEST_OWNER)),
                            pending_expiry: None,
                            pending_owner: None,
                        };
                        Ok(to_binary(&resp).unwrap())
                    }
                    _ => panic!("unexpected message"),
                }
            })
            .with_owner(TEST_MANAGER, Some(TEST_OWNER))
    }

    /// Initialize the version_control with admin and updated account_factory
    fn mock_init_with_factory(mut deps: DepsMut) -> VCResult {
        let info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(
            deps.branch(),
            mock_env(),
            info,
            InstantiateMsg {
                is_testnet: true,
                namespace_limit: 10,
            },
        )?;
        execute_as_admin(
            deps,
            ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.to_string(),
            },
        )
    }

    /// Initialize the version_control with admin as creator and test account
    fn mock_init_with_account(mut deps: DepsMut, is_testnet: bool) -> VCResult {
        let admin_info = mock_info(TEST_ADMIN, &[]);
        contract::instantiate(
            deps.branch(),
            mock_env(),
            admin_info,
            InstantiateMsg {
                is_testnet,
                namespace_limit: 10,
            },
        )?;
        execute_as_admin(
            deps.branch(),
            ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.to_string(),
            },
        )?;
        execute_as(
            deps.branch(),
            TEST_ACCOUNT_FACTORY,
            ExecuteMsg::AddAccount {
                account_id: TEST_ACCOUNT_ID,
                account_base: AccountBase {
                    manager: Addr::unchecked(TEST_MANAGER),
                    proxy: Addr::unchecked(TEST_PROXY),
                },
            },
        )
    }

    fn execute_as(deps: DepsMut, sender: &str, msg: ExecuteMsg) -> VCResult {
        contract::execute(deps, mock_env(), mock_info(sender, &[]), msg)
    }

    fn execute_as_admin(deps: DepsMut, msg: ExecuteMsg) -> VCResult {
        execute_as(deps, TEST_ADMIN, msg)
    }

    fn test_only_admin(msg: ExecuteMsg) -> VersionControlTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let _info = mock_info("not_owner", &[]);

        let res = execute_as(deps.as_mut(), "not_owner", msg);
        assert_that(&res)
            .is_err()
            .is_equal_to(VCError::Ownership(OwnershipError::NotOwner {}));

        Ok(())
    }

    mod set_admin_and_factory {
        use super::*;

        #[test]
        fn only_admin_admin() -> VersionControlTestResult {
            let msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: "new_admin".to_string(),
                expiry: None,
            });

            test_only_admin(msg)
        }

        #[test]
        fn only_admin_factory() -> VersionControlTestResult {
            let msg = ExecuteMsg::SetFactory {
                new_factory: "new_factory".to_string(),
            };
            test_only_admin(msg)
        }

        #[test]
        fn updates_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_admin = "new_admin";
            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let transfer_res = execute_as_admin(deps.as_mut(), transfer_msg).unwrap();
            assert_eq!(0, transfer_res.messages.len());

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let accept_res = execute_as(deps.as_mut(), new_admin, accept_msg).unwrap();
            assert_eq!(0, accept_res.messages.len());

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(Addr::unchecked(new_admin));

            Ok(())
        }

        #[test]
        fn updates_factory() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_factory = "new_factory";
            let msg = ExecuteMsg::SetFactory {
                new_factory: new_factory.to_string(),
            };

            let res = execute_as_admin(deps.as_mut(), msg);
            assert_that!(&res).is_ok();

            let actual_factory = FACTORY.get(deps.as_ref())?.unwrap();

            assert_that!(&actual_factory).is_equal_to(Addr::unchecked(new_factory));
            Ok(())
        }
    }

    mod claim_namespaces {
        use super::*;
        use abstract_core::objects;
        use objects::ABSTRACT_ACCOUNT_ID;

        #[test]
        fn claim_namespaces_by_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let new_namespace2 = Namespace::new("namespace2").unwrap();
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res).is_ok();
            let account_id = namespaces_info().load(&deps.storage, &new_namespace1)?;
            assert_that!(account_id).is_equal_to(TEST_ACCOUNT_ID);
            let account_id = namespaces_info().load(&deps.storage, &new_namespace2)?;
            assert_that!(account_id).is_equal_to(TEST_ACCOUNT_ID);
            Ok(())
        }

        #[test]
        fn claim_namespaces_not_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let new_namespace2 = Namespace::new("namespace2").unwrap();
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: Addr::unchecked(TEST_OTHER),
                    owner: Addr::unchecked(TEST_OWNER),
                });
            Ok(())
        }

        #[test]
        fn claim_existing_namespaces() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_namespace1 = Namespace::new("namespace1")?;
            let new_namespace2 = Namespace::new("namespace2")?;
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::NamespaceOccupied {
                    namespace: new_namespace1.to_string(),
                    id: TEST_ACCOUNT_ID,
                });
            Ok(())
        }

        #[test]
        fn cannot_claim_abstract() -> VCResult<()> {
            let mut deps = mock_dependencies();
            let account_1_manager = "manager2";
            deps.querier = mock_manager_querier()
                // add manager 2
                .with_smart_handler(account_1_manager, |msg| match from_binary(msg).unwrap() {
                    ManagerQueryMsg::Config {} => {
                        let resp = ManagerConfigResponse {
                            version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
                            module_factory_address: Addr::unchecked(TEST_MODULE_FACTORY),
                            account_id: Uint64::one(),
                            is_suspended: false,
                        };
                        Ok(to_binary(&resp).unwrap())
                    }
                    ManagerQueryMsg::Ownership {} => {
                        let resp = cw_ownable::Ownership {
                            owner: Some(Addr::unchecked(TEST_OWNER)),
                            pending_expiry: None,
                            pending_owner: None,
                        };
                        Ok(to_binary(&resp).unwrap())
                    }
                    _ => panic!("unexpected message"),
                })
                .with_owner(account_1_manager, Some(TEST_OWNER))
                .build();
            mock_init_with_account(deps.as_mut(), true)?;

            // Add account 1
            execute_as(
                deps.as_mut(),
                TEST_ACCOUNT_FACTORY,
                ExecuteMsg::AddAccount {
                    account_id: 1,
                    account_base: AccountBase {
                        manager: Addr::unchecked(account_1_manager),
                        proxy: Addr::unchecked("proxy2"),
                    },
                },
            )?;

            // Attempt to claim the abstract namespace with account 1
            let claim_abstract_msg = ExecuteMsg::ClaimNamespaces {
                account_id: 1,
                namespaces: vec![Namespace::try_from(ABSTRACT_NAMESPACE)?.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OWNER, claim_abstract_msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(VCError::NamespaceOccupied {
                    namespace: Namespace::try_from("abstract")?.to_string(),
                    id: ABSTRACT_ACCOUNT_ID,
                });
            Ok(())
        }
    }

    mod update_namespace_limit {
        use super::*;

        #[test]
        fn only_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateNamespaceLimit { new_limit: 100 };

            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner));

            Ok(())
        }

        #[test]
        fn updates_limit() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateNamespaceLimit { new_limit: 100 };

            let res = execute_as_admin(deps.as_mut(), msg);
            assert_that!(&res).is_ok();

            assert_that!(CONFIG.load(&deps.storage).unwrap().namespace_limit).is_equal_to(100);

            Ok(())
        }

        #[test]
        fn no_decrease() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateNamespaceLimit { new_limit: 0 };

            let res = execute_as_admin(deps.as_mut(), msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(VCError::DecreaseNamespaceLimit {
                    current: 10,
                    limit: 0,
                });

            Ok(())
        }
    }

    mod remove_namespaces {
        use cosmwasm_std::attr;

        use abstract_core::objects::module_reference::ModuleReference;

        use super::*;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE_ID, ModuleVersion::Version(TEST_VERSION.into()))
                .unwrap()
        }

        #[test]
        fn remove_namespaces_by_admin_or_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_namespace1 = Namespace::new("namespace1").unwrap();
            let new_namespace2 = Namespace::new("namespace2").unwrap();
            let new_namespace3 = Namespace::new("namespace3").unwrap();

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![
                    new_namespace1.to_string(),
                    new_namespace2.to_string(),
                    new_namespace3.to_string(),
                ],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // remove as admin
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let exists = namespaces_info().has(&deps.storage, &new_namespace1);
            assert_that!(exists).is_equal_to(false);

            // remove as owner
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace2.to_string(), new_namespace3.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res).is_ok();
            let exists = namespaces_info().has(&deps.storage, &new_namespace2);
            assert_that!(exists).is_equal_to(false);
            let exists = namespaces_info().has(&deps.storage, &new_namespace3);
            assert_that!(exists).is_equal_to(false);
            assert_eq!(
                res.unwrap().events[0].attributes[2],
                attr(
                    "namespaces",
                    format!(
                        "({}, {}),({}, {})",
                        new_namespace2, TEST_ACCOUNT_ID, new_namespace3, TEST_ACCOUNT_ID
                    ),
                )
            );

            Ok(())
        }

        #[test]
        fn remove_namespaces_as_other() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_namespace1 = Namespace::new("namespace1")?;
            let new_namespace2 = Namespace::new("namespace2")?;

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // remove as other
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: Addr::unchecked(TEST_OTHER),
                    owner: Addr::unchecked(TEST_OWNER),
                });
            Ok(())
        }

        #[test]
        fn remove_not_existing_namespaces() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_namespace1 = Namespace::new("namespace1")?;

            // remove as owner
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::UnknownNamespace {
                    namespace: new_namespace1.clone(),
                });

            // remove as admin
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::UnknownNamespace {
                    namespace: new_namespace1,
                });

            Ok(())
        }

        #[test]
        fn yank_orphaned_modules() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;

            // add namespaces
            let new_namespace1 = Namespace::new("namespace1")?;
            let new_namespace2 = Namespace::new("namespace2")?;
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![new_namespace1.to_string(), new_namespace2.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // first add module
            let mut new_module = test_module();
            new_module.namespace = new_namespace1.clone();
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // remove as admin
            let msg = ExecuteMsg::RemoveNamespaces {
                namespaces: vec![new_namespace1.to_string()],
            };
            execute_as(deps.as_mut(), TEST_ADMIN, msg)?;

            let module = REGISTERED_MODULES.load(&deps.storage, &new_module);
            assert_that!(&module).is_err();
            let module = YANKED_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }
    }

    mod propose_modules {
        use abstract_core::objects::module_reference::ModuleReference;
        use abstract_core::AbstractError;
        use abstract_testing::prelude::TEST_MODULE_ID;

        use super::*;

        fn test_module() -> ModuleInfo {
            ModuleInfo::from_id(TEST_MODULE_ID, ModuleVersion::Version(TEST_VERSION.into()))
                .unwrap()
        }

        // - Query latest

        #[test]
        fn add_module_by_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let mut new_module = test_module();
            new_module.namespace = Namespace::new(ABSTRACT_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn add_module_by_account_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_module = test_module();

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // try while no namespace
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::UnknownNamespace {
                    namespace: new_module.namespace.clone(),
                });

            // add namespaces
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ClaimNamespaces {
                    account_id: TEST_ACCOUNT_ID,
                    namespaces: vec![new_module.namespace.to_string()],
                },
            )?;

            // add modules
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res).is_ok();
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn try_add_module_to_approval_with_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            let contract_addr = Addr::unchecked("contract");
            // create mock with ContractInfo response for contract with admin set
            deps.querier = mock_manager_querier()
                .with_contract_admin(&contract_addr, Addr::unchecked("admin"))
                .build();

            mock_init_with_account(deps.as_mut(), false)?;
            let new_module = test_module();

            let mod_ref = ModuleReference::Adapter(contract_addr);

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), mod_ref)],
            };

            // try while no namespace
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::UnknownNamespace {
                    namespace: new_module.namespace.clone(),
                });

            // add namespaces
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ClaimNamespaces {
                    account_id: TEST_ACCOUNT_ID,
                    namespaces: vec![new_module.namespace.to_string()],
                },
            )?;

            // assert we got admin must be none error
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::AdminMustBeNone);

            Ok(())
        }

        #[test]
        fn add_module_to_approval() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), false)?;
            let new_module = test_module();

            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // try while no namespace
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::UnknownNamespace {
                    namespace: new_module.namespace.clone(),
                });

            // add namespaces
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ClaimNamespaces {
                    account_id: TEST_ACCOUNT_ID,
                    namespaces: vec![new_module.namespace.to_string()],
                },
            )?;

            // add modules
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg);
            assert_that!(&res).is_ok();
            let module = PENDING_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn approve_modules() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), false)?;
            let new_module = test_module();

            // add namespaces
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ClaimNamespaces {
                    account_id: TEST_ACCOUNT_ID,
                    namespaces: vec![new_module.namespace.to_string()],
                },
            )?;
            // add modules
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ProposeModules {
                    modules: vec![(new_module.clone(), ModuleReference::App(0))],
                },
            )?;

            let msg = ExecuteMsg::ApproveOrRejectModules {
                approves: vec![new_module.clone()],
                rejects: vec![],
            };

            // approve by owner
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            // approve by admin
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            let pending = PENDING_MODULES.has(&deps.storage, &new_module);
            assert_that!(pending).is_equal_to(false);

            Ok(())
        }

        #[test]
        fn reject_modules() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), false)?;
            let new_module = test_module();

            // add namespaces
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ClaimNamespaces {
                    account_id: TEST_ACCOUNT_ID,
                    namespaces: vec![new_module.namespace.to_string()],
                },
            )?;
            // add modules
            execute_as(
                deps.as_mut(),
                TEST_OWNER,
                ExecuteMsg::ProposeModules {
                    modules: vec![(new_module.clone(), ModuleReference::App(0))],
                },
            )?;

            let msg = ExecuteMsg::ApproveOrRejectModules {
                approves: vec![],
                rejects: vec![new_module.clone()],
            };

            // reject by owner
            let res = execute_as(deps.as_mut(), TEST_OWNER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            // reject by admin
            let res = execute_as(deps.as_mut(), TEST_ADMIN, msg);
            assert_that!(&res).is_ok();
            let exists = REGISTERED_MODULES.has(&deps.storage, &new_module);
            assert_that!(exists).is_equal_to(false);
            let pending = PENDING_MODULES.has(&deps.storage, &new_module);
            assert_that!(pending).is_equal_to(false);

            Ok(())
        }

        #[test]
        fn remove_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let rm_module = test_module();

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![rm_module.namespace.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // first add module
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;
            let module = REGISTERED_MODULES.load(&deps.storage, &rm_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));

            // then remove
            let msg = ExecuteMsg::RemoveModule {
                module: rm_module.clone(),
            };
            // as other, should fail
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            // only admin can remove modules.
            execute_as_admin(deps.as_mut(), msg)?;

            let module = REGISTERED_MODULES.load(&deps.storage, &rm_module);
            assert_that!(&module).is_err();
            Ok(())
        }

        #[test]
        fn yank_module_only_account_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let rm_module = test_module();

            // add namespaces as the account owner
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![rm_module.namespace.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // first add module as the account owner
            let add_modules_msg = ExecuteMsg::ProposeModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, add_modules_msg)?;
            let added_module = REGISTERED_MODULES.load(&deps.storage, &rm_module)?;
            assert_that!(&added_module).is_equal_to(&ModuleReference::App(0));

            // then yank the module as the other
            let msg = ExecuteMsg::YankModule { module: rm_module };
            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::AccountOwnerMismatch {
                    sender: Addr::unchecked(TEST_OTHER),
                    owner: Addr::unchecked(TEST_OWNER),
                });

            Ok(())
        }

        #[test]
        fn yank_module() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let rm_module = test_module();

            // add namespaces as the owner
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec![rm_module.namespace.to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // first add module as the owner
            let add_modules_msg = ExecuteMsg::ProposeModules {
                modules: vec![(rm_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, add_modules_msg)?;
            let added_module = REGISTERED_MODULES.load(&deps.storage, &rm_module)?;
            assert_that!(&added_module).is_equal_to(&ModuleReference::App(0));

            // then yank as owner
            let msg = ExecuteMsg::YankModule {
                module: rm_module.clone(),
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // check that the yanked module is in the yanked modules and no longer in the library
            let module = REGISTERED_MODULES.load(&deps.storage, &rm_module);
            assert_that!(&module).is_err();
            let yanked_module = YANKED_MODULES.load(&deps.storage, &rm_module)?;
            assert_that!(&yanked_module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn bad_version() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;

            // add namespaces
            let msg = ExecuteMsg::ClaimNamespaces {
                account_id: TEST_ACCOUNT_ID,
                namespaces: vec!["namespace".to_string()],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            let bad_version_module = ModuleInfo::from_id(
                TEST_MODULE_ID,
                ModuleVersion::Version("non_compliant_version".into()),
            )?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(bad_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .matches(|e| e.to_string().contains("Invalid version"));

            let latest_version_module = ModuleInfo::from_id(TEST_MODULE_ID, ModuleVersion::Latest)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(latest_version_module, ModuleReference::App(0))],
            };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Abstract(AbstractError::Assert(
                    "Module version must be set to a specific version".into(),
                )));
            Ok(())
        }

        #[test]
        fn abstract_namespace() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            let abstract_contract_id = format!("{}:{}", ABSTRACT_NAMESPACE, "test-module");
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let new_module = ModuleInfo::from_id(&abstract_contract_id, TEST_VERSION.into())?;

            // let mod_ref = ModuleReference::
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };

            // execute as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            execute_as_admin(deps.as_mut(), msg)?;
            let module = REGISTERED_MODULES.load(&deps.storage, &new_module)?;
            assert_that!(&module).is_equal_to(&ModuleReference::App(0));
            Ok(())
        }

        #[test]
        fn validates_module_info() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            let bad_modules = vec![
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: Namespace::unchecked(""),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: Namespace::unchecked(""),
                },
                ModuleInfo {
                    name: "".to_string(),
                    version: ModuleVersion::Version("0.0.1".to_string()),
                    namespace: Namespace::unchecked("test"),
                },
                ModuleInfo {
                    name: "test-module".to_string(),
                    version: ModuleVersion::Version("aoeu".to_string()),
                    namespace: Namespace::unchecked(""),
                },
            ];

            for bad_module in bad_modules {
                let msg = ExecuteMsg::ProposeModules {
                    modules: vec![(bad_module.clone(), ModuleReference::App(0))],
                };
                let res = execute_as(deps.as_mut(), TEST_OTHER, msg);
                assert_that!(&res)
                    .named(&format!("ModuleInfo validation failed for {bad_module}"))
                    .is_err()
                    .is_equal_to(&VCError::Abstract(AbstractError::FormattingError {
                        object: "module name".into(),
                        expected: "with content".into(),
                        actual: "empty".into(),
                    }));
            }

            Ok(())
        }
    }

    fn claim_test_namespace_as_owner(deps: DepsMut) -> VersionControlTestResult {
        let msg = ExecuteMsg::ClaimNamespaces {
            account_id: TEST_ACCOUNT_ID,
            namespaces: vec![TEST_NAMESPACE.to_string()],
        };
        execute_as(deps, TEST_OWNER, msg)?;
        Ok(())
    }

    mod remove_module {
        use super::*;

        #[test]
        fn test_only_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            claim_test_namespace_as_owner(deps.as_mut())?;

            // add a module as the owner
            let mut new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            new_module.namespace = Namespace::new(TEST_NAMESPACE)?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // Load the module from the library to check its presence
            assert_that!(REGISTERED_MODULES.has(&deps.storage, &new_module)).is_true();

            // now, remove the module as the admin
            let msg = ExecuteMsg::RemoveModule { module: new_module };
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg);

            assert_that!(res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));
            Ok(())
        }

        #[test]
        fn remove_from_library() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            claim_test_namespace_as_owner(deps.as_mut())?;

            // add a module as the owner
            let new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            let msg = ExecuteMsg::ProposeModules {
                modules: vec![(new_module.clone(), ModuleReference::App(0))],
            };
            execute_as(deps.as_mut(), TEST_OWNER, msg)?;

            // Load the module from the library to check its presence
            assert_that!(REGISTERED_MODULES.has(&deps.storage, &new_module)).is_true();

            // now, remove the module as the admin
            let msg = ExecuteMsg::RemoveModule {
                module: new_module.clone(),
            };
            execute_as_admin(deps.as_mut(), msg)?;

            assert_that!(REGISTERED_MODULES.has(&deps.storage, &new_module)).is_false();
            Ok(())
        }

        #[test]
        fn leaves_pending() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            claim_test_namespace_as_owner(deps.as_mut())?;

            // add a module as the owner
            let new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            PENDING_MODULES.save(deps.as_mut().storage, &new_module, &ModuleReference::App(0))?;

            // yank the module as the owner
            let msg = ExecuteMsg::RemoveModule {
                module: new_module.clone(),
            };
            let res = execute_as_admin(deps.as_mut(), msg);

            assert_that!(res)
                .is_err()
                .is_equal_to(&VCError::ModuleNotFound(new_module));
            Ok(())
        }

        #[test]
        fn remove_from_yanked() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = mock_manager_querier().build();
            mock_init_with_account(deps.as_mut(), true)?;
            claim_test_namespace_as_owner(deps.as_mut())?;

            // add a module as the owner
            let new_module = ModuleInfo::from_id(TEST_MODULE_ID, TEST_VERSION.into())?;
            YANKED_MODULES.save(deps.as_mut().storage, &new_module, &ModuleReference::App(0))?;

            // should be removed from library and added to yanked
            assert_that!(REGISTERED_MODULES.has(&deps.storage, &new_module)).is_false();
            assert_that!(YANKED_MODULES.has(&deps.storage, &new_module)).is_true();

            // now, remove the module as the admin
            let msg = ExecuteMsg::RemoveModule {
                module: new_module.clone(),
            };
            execute_as_admin(deps.as_mut(), msg)?;

            assert_that!(REGISTERED_MODULES.has(&deps.storage, &new_module)).is_false();
            assert_that!(YANKED_MODULES.has(&deps.storage, &new_module)).is_false();
            Ok(())
        }
    }

    mod register_os {
        use super::*;

        #[test]
        fn add_os() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init_with_factory(deps.as_mut())?;

            let test_core: AccountBase = AccountBase {
                manager: Addr::unchecked(TEST_MANAGER),
                proxy: Addr::unchecked(TEST_PROXY),
            };
            let msg = ExecuteMsg::AddAccount {
                account_id: 0,
                account_base: test_core.clone(),
            };

            // as other
            let res = execute_as(deps.as_mut(), TEST_OTHER, msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            // as admin
            let res = execute_as_admin(deps.as_mut(), msg.clone());
            assert_that!(&res)
                .is_err()
                .is_equal_to(&VCError::Admin(AdminError::NotAdmin {}));

            // as factory
            execute_as(deps.as_mut(), TEST_ACCOUNT_FACTORY, msg)?;

            let account = ACCOUNT_ADDRESSES.load(&deps.storage, 0)?;
            assert_that!(&account).is_equal_to(&test_core);
            Ok(())
        }
    }

    mod configure {
        use super::*;

        #[test]
        fn update_admin() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let transfer_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::TransferOwnership {
                new_owner: TEST_OTHER.to_string(),
                expiry: None,
            });

            // as other
            let transfer_res = execute_as(deps.as_mut(), TEST_OTHER, transfer_msg.clone());
            assert_that!(&transfer_res)
                .is_err()
                .is_equal_to(&VCError::Ownership(OwnershipError::NotOwner {}));

            execute_as_admin(deps.as_mut(), transfer_msg)?;

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(cw_ownable::Action::AcceptOwnership);
            let accept_res = execute_as(deps.as_mut(), TEST_OTHER, accept_msg).unwrap();
            assert_eq!(0, accept_res.messages.len());

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(Addr::unchecked(TEST_OTHER));
            Ok(())
        }

        #[test]
        fn set_factory() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::SetFactory {
                new_factory: TEST_ACCOUNT_FACTORY.into(),
            };

            test_only_admin(msg.clone())?;

            execute_as_admin(deps.as_mut(), msg)?;
            let new_factory = FACTORY.query_admin(deps.as_ref())?.admin;
            assert_that!(new_factory).is_equal_to(&Some(TEST_ACCOUNT_FACTORY.into()));
            Ok(())
        }
    }

    mod query_account_owner {
        use super::*;

        #[test]
        fn returns_account_owner() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = AbstractMockQuerierBuilder::default()
                .account(TEST_MANAGER, TEST_PROXY, 0)
                .build();
            mock_init_with_account(deps.as_mut(), true)?;

            let account_owner =
                query_account_owner(&deps.as_ref().querier, &Addr::unchecked(TEST_MANAGER), 0)?;

            assert_that!(account_owner).is_equal_to(Addr::unchecked(TEST_OWNER));
            Ok(())
        }

        #[test]
        fn no_owner_returns_err() -> VersionControlTestResult {
            let mut deps = mock_dependencies();
            deps.querier = MockQuerierBuilder::default()
                .with_contract_item(
                    TEST_MANAGER,
                    cw_storage_plus::Item::<cw_ownable::Ownership<Addr>>::new(
                        OWNERSHIP_STORAGE_KEY,
                    ),
                    &cw_ownable::Ownership {
                        owner: None,
                        pending_owner: None,
                        pending_expiry: None,
                    },
                )
                .build();
            mock_init_with_account(deps.as_mut(), true)?;

            let account_id = 0;
            let res = query_account_owner(
                &deps.as_ref().querier,
                &Addr::unchecked(TEST_MANAGER),
                account_id,
            );
            assert_that!(res)
                .is_err()
                .is_equal_to(&VCError::NoAccountOwner { account_id });
            Ok(())
        }
    }
}
