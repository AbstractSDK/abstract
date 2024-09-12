use abstract_macros::abstract_response;
use abstract_sdk::{
    execute_update_ownership, query_ownership,
    std::{account_factory::*, ACCOUNT_FACTORY},
};
use cosmwasm_std::{
    to_json_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};

use crate::{commands, error::AccountFactoryError, queries, state::*};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(ACCOUNT_FACTORY)]
pub struct AccountFactoryResponse;

pub type AccountFactoryResult<T = Response> = Result<T, AccountFactoryError>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> AccountFactoryResult {
    let config = Config {
        version_control_contract: deps.api.addr_validate(&msg.version_control_address)?,
        module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
        ans_host_contract: deps.api.addr_validate(&msg.ans_host_address)?,
    };

    cw2::set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;

    CONFIG.save(deps.storage, &config)?;
    // Set up the admin
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.admin))?;
    Ok(AccountFactoryResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> AccountFactoryResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            ans_host_contract,
            version_control_contract,
            module_factory_address,
        } => commands::execute_update_config(
            deps,
            info,
            ans_host_contract,
            version_control_contract,
            module_factory_address,
        ),
        ExecuteMsg::CreateAccount {
            governance,
            link,
            name,
            description,
            account_id,
            namespace,
            install_modules,
        } => commands::execute_create_account(
            deps,
            env,
            info,
            governance,
            name,
            description,
            link,
            namespace,
            install_modules,
            account_id,
        ),
        ExecuteMsg::UpdateOwnership(action) => {
            execute_update_ownership!(AccountFactoryResponse, deps, env, info, action)
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> AccountFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_ACCOUNT_MANAGER_MSG_ID,
            result,
            payload: _,
            gas_used: _,
        } => commands::validate_instantiated_account(deps, result),
        _ => Err(AccountFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&queries::query_config(deps)?),
        QueryMsg::Ownership {} => query_ownership!(deps),
    }
}

#[cfg(test)]
mod tests {
    use abstract_testing::prelude::*;
    use cosmwasm_std::{testing::*, Addr};
    use cw_ownable::OwnershipError;
    use speculoos::prelude::*;

    use super::*;
    use crate::test_common::*;

    type AccountFactoryTestResult = AccountFactoryResult<()>;

    fn execute_as(deps: DepsMut, sender: &Addr, msg: ExecuteMsg) -> AccountFactoryResult {
        execute(deps, mock_env(), message_info(sender, &[]), msg)
    }

    fn test_only_owner(
        deps: DepsMut,
        msg: ExecuteMsg,
        not_owner: &Addr,
    ) -> AccountFactoryTestResult {
        let res = execute_as(deps, &not_owner, msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(AccountFactoryError::Ownership(OwnershipError::NotOwner {}));

        Ok(())
    }

    mod update_config {
        use super::*;

        #[test]
        fn only_owner() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            let new_ans_host = deps.api.addr_make("test_ans_host_2");
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: Some(new_ans_host.to_string()),
                version_control_contract: None,
                module_factory_address: None,
            };

            let not_owner = deps.api.addr_make("not_owner");
            test_only_owner(deps.as_mut(), msg, &not_owner)?;

            Ok(())
        }

        #[test]
        fn update_ans_host_address() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_ans_host = deps.api.addr_make("test_ans_host_2");
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: Some(new_ans_host.to_string()),
                version_control_contract: None,
                module_factory_address: None,
            };

            execute_as(deps.as_mut(), &abstr.owner, msg)?;

            let expected_config = Config {
                version_control_contract: abstr.version_control,
                ans_host_contract: new_ans_host,
                module_factory_address: abstr.module_factory,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }

        #[test]
        fn update_version_control_address() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_version_control = deps.api.addr_make("test_version_control_2");
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: None,
                version_control_contract: Some(new_version_control.to_string()),
                module_factory_address: None,
            };

            execute_as(deps.as_mut(), &abstr.owner, msg)?;

            let expected_config = Config {
                version_control_contract: new_version_control,
                ans_host_contract: abstr.ans_host,
                module_factory_address: abstr.module_factory,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }

        #[test]
        fn update_module_factory_address() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_module_factory = deps.api.addr_make("test_module_factory_2");
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: None,
                version_control_contract: None,
                module_factory_address: Some(new_module_factory.to_string()),
            };

            execute_as(deps.as_mut(), &abstr.owner, msg)?;

            let expected_config = Config {
                version_control_contract: abstr.version_control,
                ans_host_contract: abstr.ans_host,
                module_factory_address: new_module_factory,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }

        #[test]
        fn update_all() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            let new_ans_host = deps.api.addr_make("test_ans_host_2");
            let new_version_control = deps.api.addr_make("test_version_control_2");
            let new_module_factory = deps.api.addr_make("test_module_factory_2");
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: Some(new_ans_host.to_string()),
                version_control_contract: Some(new_version_control.to_string()),
                module_factory_address: Some(new_module_factory.to_string()),
            };

            execute_as(deps.as_mut(), &abstr.owner, msg)?;

            let expected_config = Config {
                version_control_contract: new_version_control,
                ans_host_contract: new_ans_host,
                module_factory_address: new_module_factory,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }
    }

    mod update_ownership {
        use cw_ownable::Action;

        use super::*;

        #[test]
        fn only_owner() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            let new_owner = deps.api.addr_make("new_owner");
            mock_init(&mut deps)?;

            let msg = ExecuteMsg::UpdateOwnership(Action::TransferOwnership {
                new_owner: new_owner.to_string(),
                expiry: None,
            });

            let not_owner = deps.api.addr_make("not_owner");
            test_only_owner(deps.as_mut(), msg, &not_owner)?;

            Ok(())
        }

        #[test]
        fn update_owner() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            let new_admin = deps.api.addr_make("new_admin");
            mock_init(&mut deps)?;
            let abstr = AbstractMockAddrs::new(deps.api);

            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let transfer_res = execute_as(deps.as_mut(), &abstr.owner, transfer_msg);

            assert_that!(transfer_res).is_ok();

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(Action::AcceptOwnership);
            let _accept_res = execute_as(deps.as_mut(), &new_admin, accept_msg).unwrap();

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(Addr::unchecked(new_admin));

            Ok(())
        }
    }

    #[test]
    fn query_config() -> AccountFactoryTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps)?;
        let abstr = AbstractMockAddrs::new(deps.api);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_json(res).unwrap();

        assert_that!(config.version_control_contract.as_str())
            .is_equal_to(abstr.version_control.as_str());
        assert_that!(config.ans_host_contract.as_str()).is_equal_to(abstr.ans_host.as_str());
        assert_that!(config.module_factory_address.as_str())
            .is_equal_to(abstr.module_factory.as_str());

        Ok(())
    }

    #[test]
    fn query_ownership() -> AccountFactoryTestResult {
        let mut deps = mock_dependencies();
        mock_init(&mut deps)?;
        let abstr = AbstractMockAddrs::new(deps.api);

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap();
        let ownership: cw_ownable::Ownership<Addr> = from_json(res).unwrap();

        assert_that!(ownership.owner)
            .is_some()
            .is_equal_to(abstr.owner);

        Ok(())
    }

    mod migrate {
        use super::*;

        use abstract_std::AbstractError;
        use semver::Version;

        #[test]
        fn disallow_same_version() -> AccountFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = crate::migrate::migrate(deps.as_mut(), mock_env(), MigrateMsg::Migrate {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AccountFactoryError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ACCOUNT_FACTORY.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> AccountFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, ACCOUNT_FACTORY, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = crate::migrate::migrate(deps.as_mut(), mock_env(), MigrateMsg::Migrate {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AccountFactoryError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ACCOUNT_FACTORY.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> AccountFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = crate::migrate::migrate(deps.as_mut(), mock_env(), MigrateMsg::Migrate {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AccountFactoryError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: ACCOUNT_FACTORY.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> AccountFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, ACCOUNT_FACTORY, small_version)?;

            let res = crate::migrate::migrate(deps.as_mut(), mock_env(), MigrateMsg::Migrate {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
