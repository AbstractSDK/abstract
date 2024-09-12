use crate::{
    contract::{AccountResponse, AccountResult},
    error::AccountError,
    modules::update_module_addresses,
};
use abstract_sdk::{cw_helpers::AbstractAttributes, feature_objects::VersionControlContract};
use abstract_std::{
    account::{
        state::{
            AccountInfo, SuspensionStatus, ACCOUNT_ID, CONFIG, INFO, SUB_ACCOUNTS,
            SUSPENSION_STATUS,
        },
        ExecuteMsg, InternalConfigAction, UpdateSubAccountAction,
    },
    objects::{
        gov_type::GovernanceDetails,
        ownership,
        validation::{validate_description, validate_link, validate_name},
    },
};
use cosmwasm_std::{
    ensure, from_json, wasm_execute, Binary, CosmosMsg, DepsMut, MessageInfo, Response, StdError,
};

pub fn update_account_status(
    deps: DepsMut,
    info: MessageInfo,
    suspension_status: Option<bool>,
) -> Result<Response, AccountError> {
    let mut response = AccountResponse::action("update_status");

    if let Some(suspension_status) = suspension_status {
        response = update_suspension_status(deps, info, suspension_status, response)?;
    } else {
        return Err(AccountError::NoUpdates {});
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
pub fn update_internal_config(deps: DepsMut, info: MessageInfo, config: Binary) -> AccountResult {
    // deserialize the config action
    let action: InternalConfigAction =
        from_json(config).map_err(|error| AccountError::InvalidConfigAction { error })?;

    let (add, remove) = match action {
        InternalConfigAction::UpdateModuleAddresses { to_add, to_remove } => (to_add, to_remove),
        _ => {
            return Err(AccountError::InvalidConfigAction {
                error: StdError::generic_err("Unknown config action"),
            })
        }
    };

    ownership::assert_nested_owner(deps.storage, &deps.querier, &info.sender)?;
    update_module_addresses(deps, add, remove)
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

    Ok(AccountResponse::action("update_info"))
}

/// Renounce ownership of this account \
/// **WARNING**: This will lock the account, making it unusable.
pub fn remove_account_from_contracts(deps: DepsMut) -> AccountResult<Vec<CosmosMsg>> {
    let mut msgs = vec![];

    let account_id = ACCOUNT_ID.load(deps.storage)?;
    // Check for any sub accounts
    let sub_account = SUB_ACCOUNTS
        .keys(deps.storage, None, None, cosmwasm_std::Order::Ascending)
        .next()
        .transpose()?;
    ensure!(
        sub_account.is_none(),
        AccountError::RenounceWithSubAccount {}
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
