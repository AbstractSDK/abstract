use crate::{commands, error::AccountFactoryError, state::*};
use abstract_sdk::core::{
    account_factory::*,
    objects::module_version::{migrate_module_data, set_module_data},
    ACCOUNT_FACTORY,
};
use cosmwasm_std::{
    to_binary, Addr, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::{get_contract_version, set_contract_version};

use abstract_macros::abstract_response;
use abstract_sdk::{execute_update_ownership, query_ownership};
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub type AccountFactoryResult<T = Response> = Result<T, AccountFactoryError>;

#[abstract_response(ACCOUNT_FACTORY)]
pub struct AccountFactoryResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> AccountFactoryResult {
    let config = Config {
        version_control_contract: deps.api.addr_validate(&msg.version_control_address)?,
        module_factory_address: deps.api.addr_validate(&msg.module_factory_address)?,
        ans_host_contract: deps.api.addr_validate(&msg.ans_host_address)?,
        next_account_id: 0u32,
    };

    set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        ACCOUNT_FACTORY,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;

    CONFIG.save(deps.storage, &config)?;
    // Setup the admin as the creator of the contract
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(Response::new())
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
            env,
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
        } => {
            let gov_details = governance.verify(deps.api)?;
            commands::execute_create_account(deps, env, gov_details, name, description, link)
        }
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
        } => commands::after_manager_create_proxy(deps, result),
        Reply {
            id: commands::CREATE_ACCOUNT_PROXY_MSG_ID,
            result,
        } => commands::after_proxy_add_to_manager_and_set_admin(deps, result),
        _ => Err(AccountFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Ownership {} => query_ownership!(deps),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let cw_ownable::Ownership { owner, .. } = cw_ownable::get_ownership(deps.storage)?;

    let resp = ConfigResponse {
        owner: owner.unwrap_or_else(|| Addr::unchecked("")),
        version_control_contract: state.version_control_contract,
        ans_host_contract: state.ans_host_contract,
        module_factory_address: state.module_factory_address,
        next_account_id: state.next_account_id,
    };

    Ok(resp)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> StdResult<Response> {
    let version: Version = CONTRACT_VERSION.parse().unwrap();
    let storage_version: Version = get_contract_version(deps.storage)?.version.parse().unwrap();

    if storage_version < version {
        set_contract_version(deps.storage, ACCOUNT_FACTORY, CONTRACT_VERSION)?;
        migrate_module_data(
            deps.storage,
            ACCOUNT_FACTORY,
            CONTRACT_VERSION,
            None::<String>,
        )?;
    }
    Ok(Response::default())
}

#[cfg(test)]
mod tests {
    use super::*;
    use abstract_testing::prelude::*;
    use cosmwasm_std::testing::*;
    use cosmwasm_std::Addr;
    use cw_ownable::OwnershipError;
    use speculoos::prelude::*;

    type AccountFactoryTestResult = AccountFactoryResult<()>;

    fn execute_as(deps: DepsMut, sender: impl ToString, msg: ExecuteMsg) -> AccountFactoryResult {
        execute(
            deps,
            mock_env(),
            mock_info(sender.to_string().as_str(), &[]),
            msg,
        )
    }

    fn execute_as_owner(deps: DepsMut, msg: ExecuteMsg) -> AccountFactoryResult {
        execute_as(deps, TEST_ADMIN, msg)
    }

    fn mock_init(deps: DepsMut) -> AccountFactoryResult {
        instantiate(
            deps,
            mock_env(),
            mock_info(TEST_ADMIN, &[]),
            InstantiateMsg {
                version_control_address: TEST_VERSION_CONTROL.to_string(),
                ans_host_address: TEST_ANS_HOST.to_string(),
                module_factory_address: TEST_MODULE_FACTORY.to_string(),
            },
        )
    }

    fn test_only_owner(deps: DepsMut, msg: ExecuteMsg) -> AccountFactoryTestResult {
        let res = execute_as(deps, "not_admin", msg);
        assert_that!(&res)
            .is_err()
            .is_equal_to(AccountFactoryError::Ownership(OwnershipError::NotOwner {}));

        Ok(())
    }

    mod update_config {
        use super::*;
        use cosmwasm_std::Addr;

        #[test]
        fn only_owner() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_ans_host = "test_ans_host_2";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: Some(new_ans_host.to_string()),
                version_control_contract: None,
                module_factory_address: None,
            };

            test_only_owner(deps.as_mut(), msg)?;

            Ok(())
        }

        #[test]
        fn update_ans_host_address() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_ans_host = "test_ans_host_2";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: Some(new_ans_host.to_string()),
                version_control_contract: None,
                module_factory_address: None,
            };

            execute_as_owner(deps.as_mut(), msg)?;

            let expected_config = Config {
                version_control_contract: Addr::unchecked(TEST_VERSION_CONTROL),
                ans_host_contract: Addr::unchecked(new_ans_host),
                module_factory_address: Addr::unchecked(TEST_MODULE_FACTORY),
                next_account_id: 0,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }

        #[test]
        fn update_version_control_address() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_version_control = "test_version_control_2";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: None,
                version_control_contract: Some(new_version_control.to_string()),
                module_factory_address: None,
            };

            execute_as_owner(deps.as_mut(), msg)?;

            let expected_config = Config {
                version_control_contract: Addr::unchecked(new_version_control),
                ans_host_contract: Addr::unchecked(TEST_ANS_HOST),
                module_factory_address: Addr::unchecked(TEST_MODULE_FACTORY),
                next_account_id: 0,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }

        #[test]
        fn update_module_factory_address() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_module_factory = "test_module_factory_2";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: None,
                version_control_contract: None,
                module_factory_address: Some(new_module_factory.to_string()),
            };

            execute_as_owner(deps.as_mut(), msg)?;

            let expected_config = Config {
                version_control_contract: Addr::unchecked(TEST_VERSION_CONTROL),
                ans_host_contract: Addr::unchecked(TEST_ANS_HOST),
                module_factory_address: Addr::unchecked(new_module_factory),
                next_account_id: 0,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }

        #[test]
        fn update_all() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_ans_host = "test_ans_host_2";
            let new_version_control = "test_version_control_2";
            let new_module_factory = "test_module_factory_2";
            let msg = ExecuteMsg::UpdateConfig {
                ans_host_contract: Some(new_ans_host.to_string()),
                version_control_contract: Some(new_version_control.to_string()),
                module_factory_address: Some(new_module_factory.to_string()),
            };

            execute_as_owner(deps.as_mut(), msg)?;

            let expected_config = Config {
                version_control_contract: Addr::unchecked(new_version_control),
                ans_host_contract: Addr::unchecked(new_ans_host),
                module_factory_address: Addr::unchecked(new_module_factory),
                next_account_id: 0,
            };
            let actual_config: Config = CONFIG.load(deps.as_ref().storage)?;
            assert_that!(actual_config).is_equal_to(expected_config);

            Ok(())
        }
    }

    mod update_ownership {
        use super::*;
        use cw_ownable::Action;

        #[test]
        fn only_owner() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let msg = ExecuteMsg::UpdateOwnership(Action::TransferOwnership {
                new_owner: "new_owner".to_string(),
                expiry: None,
            });

            test_only_owner(deps.as_mut(), msg)?;

            Ok(())
        }

        #[test]
        fn update_owner() -> AccountFactoryTestResult {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let new_admin = "new_admin";
            // First update to transfer
            let transfer_msg = ExecuteMsg::UpdateOwnership(Action::TransferOwnership {
                new_owner: new_admin.to_string(),
                expiry: None,
            });

            let transfer_res = execute_as_owner(deps.as_mut(), transfer_msg);

            assert_that!(transfer_res).is_ok();

            // Then update and accept as the new owner
            let accept_msg = ExecuteMsg::UpdateOwnership(Action::AcceptOwnership);
            let _accept_res = execute_as(deps.as_mut(), new_admin, accept_msg).unwrap();

            assert_that!(cw_ownable::get_ownership(&deps.storage).unwrap().owner)
                .is_some()
                .is_equal_to(cosmwasm_std::Addr::unchecked(new_admin));

            Ok(())
        }
    }

    #[test]
    fn query_config() -> AccountFactoryTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Config {}).unwrap();
        let config: ConfigResponse = from_binary(&res).unwrap();

        assert_that!(config.version_control_contract.as_str()).is_equal_to(TEST_VERSION_CONTROL);
        assert_that!(config.ans_host_contract.as_str()).is_equal_to(TEST_ANS_HOST);
        assert_that!(config.module_factory_address.as_str()).is_equal_to(TEST_MODULE_FACTORY);

        Ok(())
    }

    #[test]
    fn query_ownership() -> AccountFactoryTestResult {
        let mut deps = mock_dependencies();
        mock_init(deps.as_mut())?;

        let res = query(deps.as_ref(), mock_env(), QueryMsg::Ownership {}).unwrap();
        let ownership: cw_ownable::Ownership<Addr> = from_binary(&res).unwrap();

        assert_that!(ownership.owner)
            .is_some()
            .is_equal_to(Addr::unchecked(TEST_ADMIN));

        Ok(())
    }
}
