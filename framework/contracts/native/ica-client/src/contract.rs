use crate::msg::*;
use abstract_macros::abstract_response;
use abstract_std::{
    objects::module_version::{assert_cw_contract_upgrade, migrate_module_data},
    ICA_CLIENT,
};
use cosmwasm_std::{to_json_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response};
use semver::Version;

use crate::{error::IcaClientError, queries};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) type IcaClientResult<T = Response> = Result<T, IcaClientError>;

#[abstract_response(ICA_CLIENT)]
pub(crate) struct IcaClientResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
) -> IcaClientResult {
    cw2::set_contract_version(deps.storage, ICA_CLIENT, CONTRACT_VERSION)?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(IcaClientResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> IcaClientResult {
    match msg {
        ExecuteMsg::UpdateOwnership(action) => {
            cw_ownable::update_ownership(deps, &env.block, &info.sender, action)?;
            Ok(IcaClientResponse::action("update_ownership"))
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> IcaClientResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&queries::config(deps, &env)?).map_err(Into::into),
        QueryMsg::Ownership {} => {
            to_json_binary(&cw_ownable::get_ownership(deps.storage)?).map_err(Into::into)
        }
        QueryMsg::IcaAction {
            account_address,
            chain,
            actions,
        } => to_json_binary(&queries::ica_action(
            deps,
            env,
            account_address,
            chain,
            actions,
        )?)
        .map_err(Into::into),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> IcaClientResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            abstract_sdk::cw_helpers::migrate_instantiate(deps, env, instantiate_msg, instantiate)
        }
        MigrateMsg::Migrate {} => {
            let to_version: Version = CONTRACT_VERSION.parse().unwrap();

            assert_cw_contract_upgrade(deps.storage, ICA_CLIENT, to_version)?;
            cw2::set_contract_version(deps.storage, ICA_CLIENT, CONTRACT_VERSION)?;
            migrate_module_data(deps.storage, ICA_CLIENT, CONTRACT_VERSION, None::<String>)?;
            Ok(IcaClientResponse::action("migrate"))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_common::mock_init;
    use abstract_testing::{mock_env_validated, prelude::*};
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies},
        Addr,
    };
    use cw2::CONTRACT;
    use cw_ownable::Ownership;

    #[coverage_helper::test]
    fn instantiate_works() -> IcaClientResult<()> {
        let mut deps = mock_dependencies();
        let env = mock_env_validated(deps.api);
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {};
        let info = message_info(&abstr.owner, &[]);
        let res = instantiate(deps.as_mut(), env.clone(), info, msg).unwrap();
        assert!(res.messages.is_empty());

        let ownership_resp: Ownership<Addr> =
            from_json(query(deps.as_ref(), env, QueryMsg::Ownership {})?)?;

        assert_eq!(ownership_resp.owner, Some(abstr.owner));

        // CW2
        let cw2_info = CONTRACT.load(&deps.storage).unwrap();
        assert_eq!(cw2_info.version, CONTRACT_VERSION.to_string());
        assert_eq!(cw2_info.contract, ICA_CLIENT.to_string());

        Ok(())
    }

    mod migrate {
        use super::*;

        use crate::contract;
        use abstract_std::AbstractError;

        #[coverage_helper::test]
        fn disallow_same_version() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

            assert_eq!(
                res,
                Err(IcaClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ICA_CLIENT.to_string(),
                        from: version.to_string().parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn disallow_downgrade() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, ICA_CLIENT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

            assert_eq!(
                res,
                Err(IcaClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ICA_CLIENT.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn disallow_name_change() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

            assert_eq!(
                res,
                Err(IcaClientError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: ICA_CLIENT.parse().unwrap(),
                    },
                ))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn works() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, ICA_CLIENT, small_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {})?;
            assert!(res.messages.is_empty());

            assert_eq!(
                cw2::get_contract_version(&deps.storage)?.version,
                version.to_string()
            );
            Ok(())
        }
    }
}
