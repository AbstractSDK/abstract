use abstract_sdk::std::{
    merged::{
        ExecMsg, InitMsg, QueryMsg,
    },
    objects::validation::{validate_description, validate_link, validate_name},
    proxy::state::ACCOUNT_ID,
    MANAGER,
};
use abstract_std::{
    manager::{state::ACCOUNT_MODULES, UpdateSubAccountAction},
    objects::{gov_type::GovernanceDetails, ownership},
    PROXY,
};
use cosmwasm_std::{
    ensure_eq, wasm_execute, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdError,
    StdResult,
};
use cw2::set_contract_version;

use crate::{
    error::ManagerError,
};

pub type ManagerResult<R = Response> = Result<R, ManagerError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    mut deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: InitMsg,
) -> ManagerResult {
    let a = proxy::contract::instantiate(deps.branch(), env.clone(), info.clone(), msg.proxy).unwrap();
    let b  = manager::contract::instantiate(deps.branch(), env, info, msg.manager).unwrap();
    Ok(a)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute( deps: DepsMut, env: Env, info: MessageInfo, msg: ExecMsg) -> ManagerResult {
    match msg {
        ExecMsg::Manager(manager_msg) => manager::contract::execute(deps, env, info, manager_msg).unwrap(),
        ExecMsg::Proxy(proxy_msg) => proxy::contract::execute(deps, env, info, proxy_msg).unwrap(),
    };
    Ok(Response::new())
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Manager(manager_msg) => manager::contract::query(deps, env, manager_msg).unwrap(),
        QueryMsg::Proxy(proxy_msg) => proxy::contract::query(deps, env, proxy_msg).unwrap(),
    };

    Ok(Binary::default())
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::*;
    use semver::Version;
    use speculoos::prelude::*;

    use super::*;
    use crate::{contract, test_common::mock_init};

    mod migrate {
        use abstract_std::{manager::MigrateMsg, AbstractError};
        use cw2::get_contract_version;

        use super::*;

        #[test]
        fn disallow_same_version() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MANAGER.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, MANAGER, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MANAGER.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ManagerError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: MANAGER.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> ManagerResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();

            set_contract_version(deps.as_mut().storage, MANAGER, small_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
