use crate::ibc;
use crate::{commands, error::IbcClientError, queries};
use abstract_core::objects::module_version::assert_cw_contract_upgrade;
use abstract_core::{
    ibc_client::{state::*, *},
    objects::{
        ans_host::AnsHost,
        module_version::{migrate_module_data, set_module_data},
    },
    IBC_CLIENT,
};
use abstract_macros::abstract_response;
use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdError, StdResult,
};
use cw_semver::Version;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

pub(crate) type IbcClientResult<T = Response> = Result<T, IbcClientError>;

#[abstract_response(IBC_CLIENT)]
pub(crate) struct IbcClientResponse;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> IbcClientResult {
    cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    set_module_data(
        deps.storage,
        IBC_CLIENT,
        CONTRACT_VERSION,
        &[],
        None::<String>,
    )?;
    let cfg = Config {
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
    };
    CONFIG.save(deps.storage, &cfg)?;
    ANS_HOST.save(
        deps.storage,
        &AnsHost {
            address: deps.api.addr_validate(&msg.ans_host_address)?,
        },
    )?;

    ADMIN.set(deps, Some(info.sender))?;
    Ok(IbcClientResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> IbcClientResult {
    match msg {
        ExecuteMsg::UpdateAdmin { admin } => {
            let new_admin = deps.api.addr_validate(&admin)?;
            ADMIN
                .execute_update_admin(deps, info, Some(new_admin))
                .map_err(Into::into)
        }
        ExecuteMsg::UpdateConfig {
            ans_host,
            version_control,
        } => commands::execute_update_config(deps, info, ans_host, version_control)
            .map_err(Into::into),
        ExecuteMsg::RemoteAction {
            host_chain,
            action,
            callback_request,
        } => commands::execute_send_packet(deps, env, info, host_chain, action, callback_request),
        ExecuteMsg::RegisterChainHost { chain, note, host } => {
            commands::execute_allow_chain_host(deps, env, info, chain, host, note)
        }
        ExecuteMsg::SendFunds { host_chain, funds } => {
            commands::execute_send_funds(deps, env, info, host_chain, funds).map_err(Into::into)
        }
        ExecuteMsg::Register { host_chain } => {
            commands::execute_register_account(deps, env, info, host_chain)
        }
        ExecuteMsg::RemoveHost { host_chain } => {
            commands::execute_remove_host(deps, info, host_chain).map_err(Into::into)
        }
        ExecuteMsg::Callback(c) => {
            ibc::receive_action_callback(deps, env, info, c).map_err(Into::into)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Config {} => to_binary(&queries::config(deps, env)?),
        QueryMsg::Host { chain_name } => to_binary(&queries::host(deps, chain_name)?),
        QueryMsg::Account { chain, account_id } => {
            to_binary(&queries::account(deps, chain, account_id)?)
        }
        QueryMsg::ListAccounts {} => to_binary(&queries::list_accounts(deps)?),
        QueryMsg::ListRemoteHosts {} => to_binary(&queries::list_remote_hosts(deps)?),
        QueryMsg::ListRemoteProxys {} => to_binary(&queries::list_remote_proxys(deps)?),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> IbcClientResult {
    let to_version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_cw_contract_upgrade(deps.storage, IBC_CLIENT, to_version)?;
    cw2::set_contract_version(deps.storage, IBC_CLIENT, CONTRACT_VERSION)?;
    migrate_module_data(deps.storage, IBC_CLIENT, CONTRACT_VERSION, None::<String>)?;
    Ok(IbcClientResponse::action("migrate"))
}

/// Sudo endpoint used for receiving ibc callbacks (from osmosis-ibc-hooks)
/// This is a test to verify that the ibc hooks are available on most chains
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> IbcClientResult {
    // Once we receive a sudo message, we store the ack in storage

    match msg {
        SudoMsg::IBCLifecycleComplete(IBCLifecycleComplete::IBCAck {
            channel: _,
            sequence: _,
            ack,
            success,
        }) => {
            if success {
                let mut acks = ACKS.load(deps.storage).unwrap_or(vec![]);
                acks.push(ack);
                ACKS.save(deps.storage, &acks)?;
            } else {
                return Err(IbcClientError::Std(StdError::generic_err("Ack failed")));
            }
        }
        _ => return Err(IbcClientError::Std(StdError::generic_err("Wrong variant"))),
    }

    Ok(IbcClientResponse::action("sudo-callback"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::queries::config;
    use crate::test_common::*;
    use cosmwasm_std::{
        testing::{mock_dependencies, mock_env, mock_info},
        Addr,
    };
    use cw2::CONTRACT;

    use abstract_testing::addresses::TEST_CREATOR;
    use abstract_testing::prelude::{TEST_ANS_HOST, TEST_VERSION_CONTROL};
    use speculoos::prelude::*;

    #[test]
    fn instantiate_works() {
        let mut deps = mock_dependencies();
        let msg = InstantiateMsg {
            ans_host_address: TEST_ANS_HOST.into(),
            version_control_address: TEST_VERSION_CONTROL.into(),
        };
        let info = mock_info(TEST_CREATOR, &[]);
        let res = instantiate(deps.as_mut(), mock_env(), info, msg).unwrap();
        assert_that!(res.messages).is_empty();

        // config
        let expected_config = Config {
            version_control_address: Addr::unchecked(TEST_VERSION_CONTROL),
        };

        let config_resp = config(deps.as_ref(), mock_env()).unwrap();
        assert_that!(config_resp.admin.as_str()).is_equal_to(TEST_CREATOR);

        let actual_config = CONFIG.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_config).is_equal_to(expected_config);

        // CW2
        let cw2_info = CONTRACT.load(&deps.storage).unwrap();
        assert_that!(cw2_info.version).is_equal_to(CONTRACT_VERSION.to_string());
        assert_that!(cw2_info.contract).is_equal_to(IBC_CLIENT.to_string());

        // ans host
        let actual_ans_host = ANS_HOST.load(deps.as_ref().storage).unwrap();
        assert_that!(actual_ans_host.address.as_str()).is_equal_to(TEST_ANS_HOST);
    }

    mod migrate {
        use super::*;
        use crate::contract;

        use abstract_core::AbstractError;
        use cosmwasm_std::testing::mock_dependencies;

        #[test]
        fn disallow_same_version() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IbcClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: IBC_CLIENT.to_string(),
                        from: version.to_string().parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IbcClientError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: IBC_CLIENT.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version.to_string().parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            cw2::set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(IbcClientError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: IBC_CLIENT.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> IbcClientResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            cw2::set_contract_version(deps.as_mut().storage, IBC_CLIENT, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(cw2::get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
