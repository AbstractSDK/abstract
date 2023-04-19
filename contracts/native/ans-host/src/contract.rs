use crate::commands::*;
use crate::error::AnsHostError;
use crate::queries;
use abstract_core::{
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

use abstract_macros::abstract_response;
use abstract_sdk::query_ownership;

#[abstract_response(ANS_HOST)]
pub struct AnsHostResponse;

pub type AnsHostResult<T = Response> = Result<T, AnsHostError>;

const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(feature = "export", cosmwasm_std::entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InstantiateMsg,
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

    // Set up the admin as the creator of the contract
    cw_ownable::initialize_owner(deps.storage, deps.api, Some(info.sender.as_str()))?;

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
            filter: _filter, // TODO: Implement filtering
        } => queries::query_asset_list(deps, start_after, limit),
        QueryMsg::AssetInfos { infos } => queries::query_asset_infos(deps, env, infos),
        QueryMsg::AssetInfoList {
            start_after,
            limit,
            filter: _filter, // TODO: Implement filtering
        } => queries::query_asset_info_list(deps, start_after, limit),
        QueryMsg::Contracts { entries } => {
            queries::query_contract(deps, env, entries.iter().collect())
        }
        QueryMsg::ContractList {
            start_after,
            limit,
            filter: _filter, // TODO: Implement filtering
        } => queries::query_contract_list(deps, start_after, limit),
        QueryMsg::Channels { entries: names } => {
            queries::query_channels(deps, env, names.iter().collect())
        }
        QueryMsg::ChannelList {
            start_after,
            limit,
            filter: _filter, // TODO: Implement filtering
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
pub fn migrate(deps: DepsMut, _env: Env, _msg: MigrateMsg) -> AnsHostResult {
    let version: Version = CONTRACT_VERSION.parse().unwrap();

    assert_contract_upgrade(deps.storage, ANS_HOST, version)?;
    set_contract_version(deps.storage, ANS_HOST, CONTRACT_VERSION)?;

    Ok(AnsHostResponse::action("migrate"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_common::*;
    use cosmwasm_std::testing::*;
    use speculoos::prelude::*;

    mod migrate {
        use super::*;
        use crate::contract;
        use abstract_core::AbstractError;
        use cw2::get_contract_version;

        #[test]
        fn disallow_same_version() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AnsHostError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ANS_HOST.to_string(),
                        from: version.clone(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_downgrade() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let big_version = "999.999.999";
            set_contract_version(deps.as_mut().storage, ANS_HOST, big_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AnsHostError::Abstract(
                    AbstractError::CannotDowngradeContract {
                        contract: ANS_HOST.to_string(),
                        from: big_version.parse().unwrap(),
                        to: version,
                    },
                ));

            Ok(())
        }

        #[test]
        fn disallow_name_change() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let old_version = "0.0.0";
            let old_name = "old:contract";
            set_contract_version(deps.as_mut().storage, old_name, old_version)?;

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {});

            assert_that!(res)
                .is_err()
                .is_equal_to(AnsHostError::Abstract(
                    AbstractError::ContractNameMismatch {
                        from: old_name.parse().unwrap(),
                        to: ANS_HOST.parse().unwrap(),
                    },
                ));

            Ok(())
        }

        #[test]
        fn works() -> AnsHostResult<()> {
            let mut deps = mock_dependencies();
            mock_init(deps.as_mut())?;

            let small_version = "0.0.0";
            set_contract_version(deps.as_mut().storage, ANS_HOST, small_version)?;

            let version: Version = CONTRACT_VERSION.parse().unwrap();

            let res = contract::migrate(deps.as_mut(), mock_env(), MigrateMsg {})?;
            assert_that!(res.messages).has_length(0);

            assert_that!(get_contract_version(&deps.storage)?.version)
                .is_equal_to(version.to_string());
            Ok(())
        }
    }
}
