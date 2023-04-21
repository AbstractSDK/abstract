use crate::{commands, error::ModuleFactoryError, state::*};
use abstract_core::objects::module_version::assert_contract_upgrade;
use abstract_macros::abstract_response;
use abstract_sdk::core::{module_factory::*, MODULE_FACTORY};
use cosmwasm_std::{
    to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Reply, Response, StdResult,
};
use cw2::set_contract_version;
use semver::Version;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(MODULE_FACTORY)]
pub struct ModuleFactoryResponse;

pub type ModuleFactoryResult<T = Response> = Result<T, ModuleFactoryError>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> ModuleFactoryResult {
    let config = Config {
        version_control_address: deps.api.addr_validate(&msg.version_control_address)?,
        ans_host_address: deps.api.addr_validate(&msg.ans_host_address)?,
    };

    set_contract_version(deps.storage, MODULE_FACTORY, CONTRACT_VERSION)?;
    CONFIG.save(deps.storage, &config)?;
    // Set context for after init
    CONTEXT.save(
        deps.storage,
        &Context {
            account_base: None,
            module: None,
        },
    )?;

    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;
    Ok(ModuleFactoryResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ModuleFactoryResult {
    match msg {
        ExecuteMsg::UpdateConfig {
            ans_host_address,
            version_control_address,
        } => commands::execute_update_config(
            deps,
            env,
            info,
            ans_host_address,
            version_control_address,
        ),
        ExecuteMsg::InstallModule { module, init_msg } => {
            commands::execute_create_module(deps, env, info, module, init_msg)
        }
        ExecuteMsg::UpdateFactoryBinaryMsgs { to_add, to_remove } => {
            commands::update_factory_binaries(deps, info, to_add, to_remove)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            abstract_sdk::execute_update_ownership!(ModuleFactoryResponse, deps, env, info, action)
        }
    }
}

/// This just stores the result for future query
#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn reply(deps: DepsMut, _env: Env, msg: Reply) -> ModuleFactoryResult {
    match msg {
        Reply {
            id: commands::CREATE_APP_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        Reply {
            id: commands::CREATE_STANDALONE_RESPONSE_ID,
            result,
        } => commands::register_contract(deps, result),
        _ => Err(ModuleFactoryError::UnexpectedReply {}),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_binary(&query_config(deps)?),
        QueryMsg::Context {} => to_binary(&query_context(deps)?),
        QueryMsg::Ownership {} => abstract_sdk::query_ownership!(deps),
    }
}

pub fn query_config(deps: Deps) -> StdResult<ConfigResponse> {
    let state: Config = CONFIG.load(deps.storage)?;
    let resp = ConfigResponse {
        version_control_address: state.version_control_address,
        ans_host_address: state.ans_host_address,
    };

    Ok(resp)
}

pub fn query_context(deps: Deps) -> StdResult<ContextResponse> {
    let Context {
        account_base,
        module,
    }: Context = CONTEXT.load(deps.storage)?;
    let resp = ContextResponse {
        account_base,
        module,
    };

    Ok(resp)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> ModuleFactoryResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, MODULE_FACTORY, version)?;
    set_contract_version(deps.storage, MODULE_FACTORY, CONTRACT_VERSION)?;
    Ok(ModuleFactoryResponse::action("migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::contract;
    use crate::test_common::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    mod migrate {
        use super::*;
        use abstract_core::AbstractError;
        use cw2::get_contract_version;

        #[test]
        fn disallow_same_version() -> ModuleFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ModuleFactoryError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MODULE_FACTORY.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> ModuleFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, MODULE_FACTORY, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ModuleFactoryError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: MODULE_FACTORY.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> ModuleFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(ModuleFactoryError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: MODULE_FACTORY.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> ModuleFactoryResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            set_contract_version(deps.as_mut().storage, MODULE_FACTORY, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
