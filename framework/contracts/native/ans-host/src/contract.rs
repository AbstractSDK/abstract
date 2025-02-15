use abstract_macros::abstract_response;
use abstract_sdk::query_ownership;
use abstract_std::{
    ans_host::{
        state::{Config, CONFIG, REGISTERED_DEXES},
        ExecuteMsg, InstantiateMsg, MigrateMsg, QueryMsg,
    },
    objects::module_version::assert_contract_upgrade,
    ANS_HOST,
};
use cosmwasm_std::{Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult};
use cw2::set_contract_version;
use semver::Version;

use crate::{commands::*, error::AnsHostError, queries};

#[abstract_response(ANS_HOST)]
pub struct AnsHostResponse;

pub type AnsHostResult<T = Response> = Result<T, AnsHostError>;

pub const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> AnsHostResult {
    set_contract_version(deps.storage, ANS_HOST, CONTRACT_VERSION)?;

    // Initialize the config
    CONFIG.save(
        deps.storage,
        &Config {
            next_unique_pool_id: 1.into(),
        },
    )?;

    // Initialize the dexes
    REGISTERED_DEXES.save(deps.storage, &vec![])?;

    // Set up the admin
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(&msg.admin))?;

    Ok(AnsHostResponse::action("instantiate"))
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn execute(deps: DepsMut, env: Env, info: MessageInfo, msg: ExecuteMsg) -> AnsHostResult {
    handle_message(deps, info, env, msg)
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => queries::query_config(deps),
        QueryMsg::Assets { names } => queries::query_assets(deps, env, names),
        QueryMsg::AssetList {
            start_after,
            limit,
            filter: _filter,
        } => queries::query_asset_list(deps, start_after, limit),
        QueryMsg::AssetInfos { infos } => queries::query_asset_infos(deps, env, infos),
        QueryMsg::AssetInfoList {
            start_after,
            limit,
            filter: _filter,
        } => queries::query_asset_info_list(deps, start_after, limit),
        QueryMsg::Contracts { entries } => queries::query_contract(deps, env, entries),
        QueryMsg::ContractList {
            start_after,
            limit,
            filter: _filter,
        } => queries::query_contract_list(deps, start_after, limit),
        QueryMsg::Channels { entries: names } => queries::query_channels(deps, env, names),
        QueryMsg::ChannelList {
            start_after,
            limit,
            filter: _filter,
        } => queries::query_channel_list(deps, start_after, limit),
        QueryMsg::RegisteredDexes {} => queries::query_registered_dexes(deps, env),
        QueryMsg::PoolList {
            filter,
            start_after,
            limit,
        } => queries::list_pool_entries(deps, filter, start_after, limit),

        QueryMsg::Pools { pairings: keys } => queries::query_pool_entries(deps, keys),
        QueryMsg::PoolMetadatas { ids: keys } => queries::query_pool_metadatas(deps, keys),
        QueryMsg::PoolMetadataList {
            filter,
            start_after,
            limit,
        } => queries::list_pool_metadata_entries(deps, filter, start_after, limit),
        QueryMsg::Ownership {} => query_ownership!(deps),
    }
}

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> AnsHostResult {
    match msg {
        MigrateMsg::Instantiate(instantiate_msg) => {
            abstract_sdk::cw_helpers::migrate_instantiate(deps, env, instantiate_msg, instantiate)
        }
        MigrateMsg::Migrate {} => {
            let version: Version = CONTRACT_VERSION.parse().unwrap();

            assert_contract_upgrade(deps.storage, ANS_HOST, version)?;
            set_contract_version(deps.storage, ANS_HOST, CONTRACT_VERSION)?;

            Ok(AnsHostResponse::action("migrate"))
        }
    }
}

#[cfg(test)]
mod tests {
    use cosmwasm_std::testing::*;

    use super::*;
    use crate::test_common::*;

    mod migrate {
        use abstract_std::AbstractError;
        use abstract_testing::mock_env_validated;
        use cw2::get_contract_version;

        use super::*;
        use crate::contract;

        #[coverage_helper::test]
        fn disallow_same_version() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

            assert_eq!(
                res,
                Err(AnsHostError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ANS_HOST.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn disallow_downgrade() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, ANS_HOST, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

            assert_eq!(
                res,
                Err(AnsHostError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ANS_HOST.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn disallow_name_change() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);
            mock_init(&mut deps)?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {});

            assert_eq!(
                res,
                Err(AnsHostError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: ANS_HOST.parse().unwrap(),
                    },
                ))
            );

            Ok(())
        }

        #[coverage_helper::test]
        fn works() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            let env = mock_env_validated(deps.api);

            mock_init(&mut deps)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let small_version = Version {
                minor: version.minor - 1,
                ..version.clone()
            }
            .to_string();
            set_contract_version(deps.as_mut().storage, ANS_HOST, small_version)?;

            let res = contract::migrate(deps.as_mut(), env, MigrateMsg::Migrate {})?;
            assert_eq!(res.messages.len(), 0);

            assert_eq!(
                get_contract_version(&deps.storage)?.version,
                version.to_string()
            );
            Ok(())
        }
    }
}
