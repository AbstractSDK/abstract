use cosmwasm_std::{QuerierWrapper, StdResult};
use wyndex::factory::PairsResponse;
use wyndex::factory::QueryMsg;

/// Returns a list of all pairs
pub fn query_pairs(
    querier: &QuerierWrapper,
    factory_contract: impl Into<String>,
) -> StdResult<PairsResponse> {
    querier.query_wasm_smart(
        factory_contract,
        &QueryMsg::Pairs {
            start_after: None,
            limit: Some(u32::MAX),
        },
    )
}

/// Returns whether the given address is an LP staking contract of the factory.
pub fn query_validate_staking_address(
    querier: &QuerierWrapper,
    factory_contract: impl Into<String>,
    staking_address: impl Into<String>,
) -> StdResult<bool> {
    querier.query_wasm_smart(
        factory_contract,
        &QueryMsg::ValidateStakingAddress {
            address: staking_address.into(),
        },
    )
}
