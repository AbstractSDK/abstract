use abstract_macros::abstract_response;
use abstract_sdk::{
    feature_objects::{AnsHost, RegistryContract},
    std::{module_factory::*, MODULE_FACTORY},
};
use abstract_std::objects::{
    module::{ModuleInfo, Monetization},
    module_version::assert_contract_upgrade,
};
use cosmwasm_std::{
    to_json_binary, Binary, Coins, Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult,
};
use cw2::set_contract_version;
use semver::Version;

use crate::{commands, error::ModuleFactoryError};

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[abstract_response(MODULE_FACTORY)]
pub struct ModuleFactoryResponse;

pub type ModuleFactoryResult<T = Response> = Result<T, ModuleFactoryError>;

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> ModuleFactoryResult {
    set_contract_version(deps.storage, MODULE_FACTORY, CONTRACT_VERSION)?;

    // Set up the admin
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.admin))?;

    Ok(ModuleFactoryResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> ModuleFactoryResult {
    match msg {
        ExecuteMsg::InstallModules { modules, salt } => {
            commands::execute_create_modules(deps, env, info, modules, salt)
        }
        ExecuteMsg::UpdateOwnership(action) => {
            abstract_sdk::execute_update_ownership!(ModuleFactoryResponse, deps, env, info, action)
        }
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&query_config(deps, &env)?),
        QueryMsg::SimulateInstallModules { modules } => {
            to_json_binary(&query_simulate_install_modules(deps, &env, modules)?)
        }
        QueryMsg::Ownership {} => abstract_sdk::query_ownership!(deps),
    }
}

pub fn query_config(deps: Deps, env: &Env) -> StdResult<ConfigResponse> {
    let resp = ConfigResponse {
        registry_address: RegistryContract::new(deps.api, env)
            .map_err(|e| StdError::generic_err(e.to_string()))?
            .address,
        ans_host_address: AnsHost::new(deps.api, env)
            .map_err(|e| StdError::generic_err(e.to_string()))?
            .address,
    };

    Ok(resp)
}

pub fn query_simulate_install_modules(
    deps: Deps,
    env: &Env,
    modules: Vec<ModuleInfo>,
) -> StdResult<SimulateInstallModulesResponse> {
    let registry =
        RegistryContract::new(deps.api, env).map_err(|e| StdError::generic_err(e.to_string()))?;

    let module_responses = registry
        .query_modules_configs(modules, &deps.querier)
        .map_err(|e| cosmwasm_std::StdError::generic_err(e.to_string()))?;

    let mut coins = Coins::default();
    let mut install_funds = vec![];
    let mut init_funds = vec![];
    for module in module_responses {
        if let Monetization::InstallFee(fee) = module.config.monetization {
            coins.add(fee.fee())?;
            install_funds.push((module.module.info.id(), fee.fee()))
        }
        if !module.config.instantiation_funds.is_empty() {
            init_funds.push((
                module.module.info.id(),
                module.config.instantiation_funds.clone(),
            ));

            for init_coin in module.config.instantiation_funds {
                coins.add(init_coin)?;
            }
        }
    }
    let resp = SimulateInstallModulesResponse {
        total_required_funds: coins.into_vec(),
        monetization_funds: install_funds,
        initialization_funds: init_funds,
    };
    Ok(resp)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> ModuleFactoryResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            abstract_sdk::cw_helpers::migrate_instantiate(deps, env, instantiate_msg, instantiate)
        }
        MigrateMsg::Migrate {} => {
            let version: Version = CONTRACT_VERSION.parse().unwrap();

            assert_contract_upgrade(deps.storage, MODULE_FACTORY, version)?;
            set_contract_version(deps.storage, MODULE_FACTORY, CONTRACT_VERSION)?;

            Ok(ModuleFactoryResponse::action("migrate"))
        }
    }
}

#[cfg(test)]
mod tests {
    use assertor::*;
    use cosmwasm_std::testing::*;

    use super::*;
    use crate::{contract, test_common::*};

    mod migrate {
        use abstract_std::AbstractError;
        use abstract_testing::mock_env_validated;
        use cw2::get_contract_version;

        use super::*;

        #[test]
        fn disallow_same_version() -> ModuleFactoryResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, MODULE_FACTORY, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

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
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            set_contract_version(deps.as_mut().storage, MODULE_FACTORY, small_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
