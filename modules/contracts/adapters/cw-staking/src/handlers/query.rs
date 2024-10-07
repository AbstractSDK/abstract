use abstract_adapter::sdk::features::{AbstractNameService, AbstractRegistryAccess};
use abstract_staking_standard::{msg::StakingQueryMsg, CwStakingError};
use cosmwasm_std::{to_json_binary, Binary, Deps, Env, StdError};

use crate::{
    contract::{CwStakingAdapter, StakingResult},
    resolver::{self, is_over_ibc},
};
/// Handle queries related to staking
pub fn query_handler(
    deps: Deps,
    env: Env,
    module: &CwStakingAdapter,
    msg: StakingQueryMsg,
) -> StakingResult<Binary> {
    let name_service = module.name_service(deps, &env);
    let ans_host = name_service.host();
    let registry_contract = module.abstract_registry(deps, &env)?;

    match msg {
        StakingQueryMsg::Info {
            provider,
            staking_tokens,
        } => {
            // if provider is on an app-chain, error
            let (local_provider_name, is_over_ibc) = is_over_ibc(&env, &provider)?;
            if is_over_ibc {
                Err(CwStakingError::IbcQueryNotSupported)
            } else {
                // the query can be executed on the local chain
                let mut provider = resolver::resolve_local_provider(&local_provider_name)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                provider.fetch_data(
                    deps,
                    env,
                    None,
                    ans_host,
                    registry_contract,
                    staking_tokens,
                )?;
                Ok(to_json_binary(&provider.query_info(&deps.querier)?)?)
            }
        }
        StakingQueryMsg::Staked {
            provider,
            staker_address,
            stakes,
            unbonding_period,
        } => {
            let staking_tokens = stakes.clone();
            // if provider is on an app-chain, error
            let (local_provider_name, is_over_ibc) = is_over_ibc(&env, &provider)?;
            if is_over_ibc {
                Err(CwStakingError::IbcQueryNotSupported)
            } else {
                // the query can be executed on the local chain
                let mut provider = resolver::resolve_local_provider(&local_provider_name)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                provider.fetch_data(
                    deps,
                    env,
                    None,
                    ans_host,
                    registry_contract,
                    staking_tokens,
                )?;
                Ok(to_json_binary(&provider.query_staked(
                    &deps.querier,
                    deps.api.addr_validate(&staker_address)?,
                    stakes,
                    unbonding_period,
                )?)?)
            }
        }
        StakingQueryMsg::Unbonding {
            provider,
            staking_tokens,
            staker_address,
        } => {
            // if provider is on an app-chain, error
            let (local_provider_name, is_over_ibc) = is_over_ibc(&env, &provider)?;
            if is_over_ibc {
                Err(CwStakingError::IbcQueryNotSupported)
            } else {
                // the query can be executed on the local chain
                let mut provider = resolver::resolve_local_provider(&local_provider_name)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                provider.fetch_data(
                    deps,
                    env,
                    None,
                    ans_host,
                    registry_contract,
                    staking_tokens,
                )?;
                Ok(to_json_binary(&provider.query_unbonding(
                    &deps.querier,
                    deps.api.addr_validate(&staker_address)?,
                )?)?)
            }
        }
        StakingQueryMsg::RewardTokens {
            provider,
            staking_tokens,
        } => {
            // if provider is on an app-chain, error
            let (local_provider_name, is_over_ibc) = is_over_ibc(&env, &provider)?;
            if is_over_ibc {
                Err(CwStakingError::IbcQueryNotSupported)
            } else {
                // the query can be executed on the local chain
                let mut provider = resolver::resolve_local_provider(&local_provider_name)
                    .map_err(|e| StdError::generic_err(e.to_string()))?;
                provider.fetch_data(
                    deps,
                    env,
                    None,
                    ans_host,
                    registry_contract,
                    staking_tokens,
                )?;
                Ok(to_json_binary(&provider.query_rewards(&deps.querier)?)?)
            }
        }
    }
}
