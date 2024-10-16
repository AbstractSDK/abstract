use crate::{
    contract::{AccountResponse, AccountResult},
    error::AbstractXionError,
    modules::{_update_whitelisted_modules, update_module_addresses},
};
use abstract_sdk::cw_helpers::AbstractAttributes;
use abstract_std::{
    account::{
        state::{AccountInfo, SuspensionStatus, INFO, SUSPENSION_STATUS},
        InternalConfigAction,
    },
    objects::{
        ownership,
        validation::{validate_description, validate_link, validate_name},
    },
};
use cosmwasm_std::{Addr, DepsMut, MessageInfo, Response, StdError};

pub fn update_account_status(
    deps: DepsMut,
    info: MessageInfo,
    suspension_status: Option<bool>,
) -> Result<Response, AbstractXionError> {
    let mut response = AccountResponse::action("update_status");

    if let Some(suspension_status) = suspension_status {
        response = update_suspension_status(deps, info, suspension_status, response)?;
    } else {
        return Err(AbstractXionError::NoUpdates {});
    }

    Ok(response)
}

pub fn update_suspension_status(
    deps: DepsMut,
    info: MessageInfo,
    is_suspended: SuspensionStatus,
    response: Response,
) -> AccountResult {
    // only owner can update suspension status
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    SUSPENSION_STATUS.save(deps.storage, &is_suspended)?;

    Ok(response.add_abstract_attributes(vec![("is_suspended", is_suspended.to_string())]))
}

/// Allows the owner to manually update the internal configuration of the account.
/// This can be used to unblock the account and its modules in case of a bug/lock on the account.
pub fn update_internal_config(
    deps: DepsMut,
    info: MessageInfo,
    action: InternalConfigAction,
) -> AccountResult {
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    match action {
        InternalConfigAction::UpdateModuleAddresses { to_add, to_remove } => {
            let api = deps.api;

            // validate addresses
            let add: Result<Vec<(String, Addr)>, _> = to_add
                .into_iter()
                .map(|(a, b)| {
                    let addr = api.addr_validate(&b)?;
                    Ok::<(String, Addr), StdError>((a, addr))
                })
                .collect();
            let add = add?;

            update_module_addresses(deps, add, to_remove)
        }
        InternalConfigAction::UpdateWhitelist { to_add, to_remove } => {
            let module_addresses_to_add = to_add
                .into_iter()
                .map(|str_addr| deps.api.addr_validate(&str_addr))
                .collect::<Result<Vec<Addr>, _>>()?;
            let module_addresses_to_remove = to_remove
                .into_iter()
                .map(|str_addr| deps.api.addr_validate(&str_addr))
                .collect::<Result<Vec<Addr>, _>>()?;

            _update_whitelisted_modules(
                deps.storage,
                module_addresses_to_add,
                module_addresses_to_remove,
            )?;

            Ok(AccountResponse::action("update_whitelist"))
        }
        _ => Err(AbstractXionError::InvalidConfigAction {
            error: StdError::generic_err("Unknown config action"),
        }),
    }
}

/// Update the Account information
pub fn update_info(
    deps: DepsMut,
    info: MessageInfo,
    name: Option<String>,
    description: Option<String>,
    link: Option<String>,
) -> AccountResult {
    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;

    let mut info: AccountInfo = INFO.may_load(deps.storage)?.unwrap_or_default();
    if let Some(name) = name {
        validate_name(&name)?;
        info.name = Some(name);
    }
    validate_description(description.as_deref())?;
    info.description = description;
    validate_link(link.as_deref())?;
    info.link = link;
    INFO.save(deps.storage, &info)?;

    Ok(AccountResponse::action("update_info"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::test_only_owner;
    use crate::test_common::{execute_as, mock_init};
    use abstract_std::account::ExecuteMsg;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr, StdError};
    use ownership::{GovAction, GovOwnershipError, GovernanceDetails};
    use speculoos::prelude::*;

    mod set_owner_and_gov_type {

        use super::*;

        #[coverage_helper::test]
        fn only_owner() -> anyhow::Result<()> {
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

        #[coverage_helper::test]
        fn validates_new_owner_address() -> anyhow::Result<()> {
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

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(res).is_err().matches(|err| {
                matches!(
                    err,
                    AbstractXionError::Ownership(GovOwnershipError::Abstract(
                        abstract_std::AbstractError::Std(StdError::GenericErr { .. })
                    ))
                )
            });
            Ok(())
        }

        #[coverage_helper::test]
        fn updates_owner() -> anyhow::Result<()> {
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

            let res = execute_as(&mut deps, &owner, set_owner_msg);
            assert_that!(&res).is_ok();

            let accept_msg = ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership);
            execute_as(&mut deps, &new_owner, accept_msg)?;

            let actual_owner = ownership::get_ownership(&deps.storage)?.owner;

            assert_that!(&actual_owner).is_equal_to(GovernanceDetails::Monarchy {
                monarch: Addr::unchecked(new_owner),
            });

            Ok(())
        }

        #[coverage_helper::test]
        fn updates_governance_type() -> anyhow::Result<()> {
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

            execute_as(&mut deps, &owner, msg)?;

            let ownership = ownership::get_ownership(deps.as_ref().storage)?;
            assert_that!(ownership
                .owner
                .owner_address(&deps.as_ref().querier)
                .unwrap()
                .to_string())
            .is_equal_to(owner.to_string());

            let accept_msg = ExecuteMsg::UpdateOwnership(ownership::GovAction::AcceptOwnership);
            execute_as(&mut deps, &new_gov, accept_msg)?;

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

    mod update_info {
        use abstract_std::objects::validation::ValidationError;

        use super::*;

        #[coverage_helper::test]
        fn only_owner() -> anyhow::Result<()> {
            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: None,
            };

            test_only_owner(msg)
        }
        // integration tests

        #[coverage_helper::test]
        fn updates() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let name = "new name";
            let description = "new description";
            let link = "http://a.be";

            let msg = ExecuteMsg::UpdateInfo {
                name: Some(name.to_string()),
                description: Some(description.to_string()),
                link: Some(link.to_string()),
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(&res).is_ok();

            let info = INFO.load(deps.as_ref().storage)?;

            assert_that!(&info.name.unwrap()).is_equal_to(name.to_string());
            assert_that!(&info.description.unwrap()).is_equal_to(description.to_string());
            assert_that!(&info.link.unwrap()).is_equal_to(link.to_string());

            Ok(())
        }

        #[coverage_helper::test]
        fn removals() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let prev_name = "name".to_string();
            INFO.save(
                deps.as_mut().storage,
                &AccountInfo {
                    name: Some(prev_name.clone()),
                    description: Some("description".to_string()),
                    link: Some("link".to_string()),
                },
            )?;

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: None,
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(&res).is_ok();

            let info = INFO.load(deps.as_ref().storage)?;

            assert_that!(&info.name.unwrap()).is_equal_to(&prev_name);
            assert_that!(&info.description).is_none();
            assert_that!(&info.link).is_none();

            Ok(())
        }

        #[coverage_helper::test]
        fn validates_name() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateInfo {
                name: Some("".to_string()),
                description: None,
                link: None,
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    AbstractXionError::Validation(ValidationError::TitleInvalidShort(_))
                )
            });

            let msg = ExecuteMsg::UpdateInfo {
                name: Some("a".repeat(65)),
                description: None,
                link: None,
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    AbstractXionError::Validation(ValidationError::TitleInvalidLong(_))
                )
            });

            Ok(())
        }

        #[coverage_helper::test]
        fn validates_link() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: Some("aoeu".to_string()),
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    AbstractXionError::Validation(ValidationError::LinkInvalidShort(_))
                )
            });

            let msg = ExecuteMsg::UpdateInfo {
                name: None,
                description: None,
                link: Some("a".repeat(129)),
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(&res).is_err().matches(|e| {
                matches!(
                    e,
                    AbstractXionError::Validation(ValidationError::LinkInvalidLong(_))
                )
            });

            Ok(())
        }
    }

    mod update_suspension_status {
        use super::*;

        #[coverage_helper::test]
        fn only_owner() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            test_only_owner(msg)
        }

        #[coverage_helper::test]
        fn exec_fails_when_suspended() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            let res = execute_as(&mut deps, &owner, msg);
            assert_that!(res).is_ok();
            let actual_is_suspended = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that!(&actual_is_suspended).is_true();

            let update_info_msg = ExecuteMsg::UpdateInfo {
                name: Some("asonetuh".to_string()),
                description: None,
                link: None,
            };

            let res = execute_as(&mut deps, &owner, update_info_msg);

            assert_that!(&res)
                .is_err()
                .is_equal_to(AbstractXionError::AccountSuspended {});

            Ok(())
        }

        #[coverage_helper::test]
        fn suspend_account() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(true),
            };

            let res = execute_as(&mut deps, &owner, msg);

            assert_that!(&res).is_ok();
            let actual_is_suspended = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that!(&actual_is_suspended).is_true();
            Ok(())
        }

        #[coverage_helper::test]
        fn unsuspend_account() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateStatus {
                is_suspended: Some(false),
            };

            let res = execute_as(&mut deps, &owner, msg);

            assert_that!(&res).is_ok();
            let actual_status = SUSPENSION_STATUS.load(&deps.storage).unwrap();
            assert_that!(&actual_status).is_false();
            Ok(())
        }
    }

    mod update_internal_config {
        use abstract_std::account::InternalConfigAction;
        use ownership::GovOwnershipError;

        use crate::modules::WHITELIST_SIZE_LIMIT;

        use super::*;

        #[coverage_helper::test]
        fn only_account_owner() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            let msg =
                ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateModuleAddresses {
                    to_add: vec![],
                    to_remove: vec![],
                });

            let bad_sender = deps.api.addr_make("not_account_owner");
            let res = execute_as(&mut deps, &bad_sender, msg.clone());

            assert_that!(&res)
                .is_err()
                .is_equal_to(AbstractXionError::Ownership(GovOwnershipError::NotOwner));

            let vc_res = execute_as(&mut deps, &abstr.registry, msg.clone());
            assert_that!(&vc_res).is_err();

            let owner_res = execute_as(&mut deps, &owner, msg);
            assert_that!(&owner_res).is_ok();

            Ok(())
        }

        #[coverage_helper::test]
        fn whitelist_size_limit() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            // One too many
            let mut to_add: Vec<String> = (0..WHITELIST_SIZE_LIMIT + 1)
                .map(|i| deps.api.addr_make(&format!("white_list_{i}")).to_string())
                .collect();
            let too_many_msg =
                ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                    to_add: to_add.clone(),
                    to_remove: vec![],
                });
            let too_many = execute_as(&mut deps, &owner, too_many_msg).unwrap_err();
            assert_eq!(too_many, AbstractXionError::ModuleLimitReached {});

            // Exact amount
            to_add.pop();
            let exactly_limit_msg =
                ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                    to_add: to_add.clone(),
                    to_remove: vec![],
                });
            let white_list_add = execute_as(&mut deps, &owner, exactly_limit_msg);
            assert!(white_list_add.is_ok());

            // Can't add after hitting limit
            let to_add = vec![deps.api.addr_make("over_limit").to_string()];
            let module_limit_reached = execute_as(
                &mut deps,
                &owner,
                ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                    to_add,
                    to_remove: vec![],
                }),
            )
            .unwrap_err();
            assert_eq!(
                module_limit_reached,
                AbstractXionError::ModuleLimitReached {}
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn whitelist_duplicates() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            // duplicate after add
            let to_add: Vec<String> = vec![deps.api.addr_make("module").to_string()];
            let msg = ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                to_add: to_add.clone(),
                to_remove: vec![],
            });
            execute_as(&mut deps, &owner, msg.clone()).unwrap();

            let duplicate_err = execute_as(&mut deps, &owner, msg).unwrap_err();
            assert_eq!(
                duplicate_err,
                AbstractXionError::AlreadyWhitelisted(to_add[0].clone())
            );

            // duplicate inside add
            let to_add: Vec<String> = vec![
                deps.api.addr_make("module2").to_string(),
                deps.api.addr_make("module2").to_string(),
            ];
            let msg = ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                to_add: to_add.clone(),
                to_remove: vec![],
            });
            let duplicate_err = execute_as(&mut deps, &owner, msg).unwrap_err();
            assert_eq!(
                duplicate_err,
                AbstractXionError::AlreadyWhitelisted(to_add[0].clone())
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn whitelist_remove() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;

            mock_init(&mut deps)?;

            // Add and remove same
            let to_add: Vec<String> = vec![deps.api.addr_make("module").to_string()];
            let msg = ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                to_add: to_add.clone(),
                to_remove: to_add.clone(),
            });
            let no_changes = execute_as(&mut deps, &owner, msg.clone());
            assert!(no_changes.is_ok());

            // Remove not whitelisted
            let to_remove: Vec<String> = vec![deps.api.addr_make("module").to_string()];
            let msg = ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                to_add: vec![],
                to_remove,
            });
            let not_whitelisted = execute_as(&mut deps, &owner, msg.clone()).unwrap_err();
            assert_eq!(not_whitelisted, AbstractXionError::NotWhitelisted {});

            // Remove same twice
            let to_add: Vec<String> = vec![deps.api.addr_make("module").to_string()];
            let to_remove: Vec<String> = vec![
                deps.api.addr_make("module").to_string(),
                deps.api.addr_make("module").to_string(),
            ];
            let msg = ExecuteMsg::UpdateInternalConfig(InternalConfigAction::UpdateWhitelist {
                to_add: to_add.clone(),
                to_remove: to_remove.clone(),
            });
            let not_whitelisted = execute_as(&mut deps, &owner, msg.clone()).unwrap_err();
            assert_eq!(not_whitelisted, AbstractXionError::NotWhitelisted {});

            Ok(())
        }
    }

    mod update_ownership {
        use abstract_sdk::namespaces::OWNERSHIP_STORAGE_KEY;
        use cw_storage_plus::Item;

        use super::*;

        #[coverage_helper::test]
        fn allows_ownership_acceptance() -> anyhow::Result<()> {
            let mut deps = mock_dependencies();
            let abstr = AbstractMockAddrs::new(deps.api);
            let owner = abstr.owner;
            mock_init(&mut deps)?;

            let pending_owner = deps.api.addr_make("not_owner");
            // mock pending owner
            Item::new(OWNERSHIP_STORAGE_KEY).save(
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

            execute_as(&mut deps, &pending_owner, msg)?;

            Ok(())
        }
    }
}
