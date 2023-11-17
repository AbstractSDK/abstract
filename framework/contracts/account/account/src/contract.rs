use crate::error::AccountError;

use abstract_sdk::core::account::{ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response};

pub type AccountResult<R = Response> = Result<R, AccountError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AccountResult {
    // let version: Version = CONTRACT_VERSION.parse().unwrap();

    // assert_contract_upgrade(deps.storage, MANAGER, version)?;
    // set_contract_version(deps.storage, MANAGER, CONTRACT_VERSION)?;
    Ok(Response::new())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> AccountResult {
    let resp =
        manager::contract::instantiate(deps.branch(), env.clone(), info.clone(), msg.manager)?;
    let _resp2 = proxy::contract::instantiate(deps, env, info, msg.proxy)?;
    Ok(resp)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> AccountResult {
    match msg {
        ExecuteMsg::Proxy(msg) => {
            proxy::contract::execute(deps, env, info, msg).map_err(Into::into)
        }
        ExecuteMsg::Manager(msg) => {
            manager::contract::execute(deps, env, info, msg).map_err(Into::into)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> AccountResult<Binary> {
    match msg {
        QueryMsg::Proxy(msg) => proxy::contract::query(deps, env, msg).map_err(Into::into),
        QueryMsg::Manager(msg) => manager::contract::query(deps, env, msg).map_err(Into::into),
    }
}

// #[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
// pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> AccountResult {
//     match msg.id {
//         commands::REGISTER_MODULES_DEPENDENCIES => {
//             commands::register_dependencies(deps, msg.result)
//         }
//         _ => Err(AccountError::UnexpectedReply {}),
//     }
// }

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    use crate::test_common::mock_init;

    mod migrate {
        use super::*;
        use abstract_core::AbstractError;
        use cw2::get_contract_version;

        #[test]
        fn disallow_same_version() -> AccountResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AccountError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MANAGER.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> AccountResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, MANAGER, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AccountError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MANAGER.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> AccountResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AccountError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: MANAGER.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> AccountResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            set_contract_version(deps.as_mut().storage, MANAGER, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
