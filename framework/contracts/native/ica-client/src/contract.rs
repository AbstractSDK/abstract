use crate::msg::*;
use abstract_macros::abstract_response;
use abstract_sdk::feature_objects::VersionControlContract;
use abstract_std::{
    ica_client::state::{Config, CONFIG},
    objects::{
        ans_host::AnsHost,
        module_version::{assert_cw_contract_upgrade, migrate_module_data},
    },
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
    msg: InstantiateMsg,
) -> IcaClientResult {
    cw2::set_contract_version(deps.storage, ICA_CLIENT, CONTRACT_VERSION)?;
    let cfg = Config {
        version_control: VersionControlContract::new(
            deps.api.addr_validate(&msg.version_control_address)?,
        ),
        ans_host: AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    };
    CONFIG.save(deps.storage, &cfg)?;

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
        QueryMsg::Config {} => to_json_binary(&queries::config(deps)?).map_err(Into::into),
        QueryMsg::Ownership {} => {
            to_json_binary(&cw_ownable::get_ownership(deps.storage)?).map_err(Into::into)
        }
        QueryMsg::IcaAction {
            proxy_address,
            chain,
            actions,
        } => to_json_binary(&queries::ica_action(
            deps,
            env,
            proxy_address,
            chain,
            actions,
        )?)
        .map_err(Into::into),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> IcaClientResult {
    let to_version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_cw_contract_upgrade(deps.storage, ICA_CLIENT, to_version)?;
    cw2::set_contract_version(deps.storage, ICA_CLIENT, CONTRACT_VERSION)?;
    migrate_module_data(deps.storage, ICA_CLIENT, CONTRACT_VERSION, None::<String>)?;
    Ok(IcaClientResponse::action("migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::test_common::mock_init;
    use abstract_testing::prelude::*;
    use cosmwasm_std::{
        from_json,
        testing::{message_info, mock_dependencies, mock_env},
        Addr,
    };
    use cw2::CONTRACT;
    use cw_ownable::Ownership;
    use speculoos::prelude::*;

    #[test]
    fn instantiate_works() -> IcaClientResult<()> {
        let mut deps = mock_dependencies();
        let abstr = AbstractMockAddrs::new(deps.api);
        let msg = InstantiateMsg {
            ans_host_address: abstr.ans_host.to_string(),
            version_control_address: abstr.version_control.to_string(),
        };
        let info = message_info(&abstr.owner, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        // config
        let expected_config = Config {
            version_control: VersionControlContract::new(abstr.version_control),
            ans_host: AnsHost::new(abstr.ans_host),
        };

        let ownership_resp: Ownership<Addr> =
            from_json(query(deps.as_ref(), mock_env(), QueryMsg::Ownership {})?)?;

        assert_eq!(ownership_resp.owner, Some(abstr.owner));

        let actual_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_config).is_equal_to(expected_config);

        // CW2
        let cw2_info = CONTRACT.load(&deps.storage).unwrap();
        assert_that!(cw2_info.version).is_equal_to(CONTRACT_VERSION.to_string());
        assert_that!(cw2_info.contract).is_equal_to(ICA_CLIENT.to_string());

        Ok(())
    }

    mod migrate {
        use super::*;

        use crate::contract;
        use abstract_std::AbstractError;

        #[test]
        fn disallow_same_version() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IcaClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ICA_CLIENT.to_string(),
                        from: version.to_string().parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, ICA_CLIENT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IcaClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ICA_CLIENT.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IcaClientError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: ICA_CLIENT.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> IcaClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            cw2::set_contract_version(deps.as_mut().storage, ICA_CLIENT, small_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
