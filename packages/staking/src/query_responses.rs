use cosmwasm_std::{Addr, Uint128};

use cw_asset::AssetInfo;

use cw_utils::{Duration, Expiration};

#[cosmwasm_schema::cw_serde]
pub struct StakingInfoResponse {
    pub staking_contract_address: Addr,
    pub staking_token: AssetInfo,
    pub unbonding_periods: Option<Vec<Duration>>,
    pub max_claims: Option<u32>,
}

#[cosmwasm_schema::cw_serde]
pub struct StakeResponse {
    pub amount: Uint128,
}

#[cosmwasm_schema::cw_serde]
pub struct RewardTokensResponse {
    pub tokens: Vec<AssetInfo>,
}

#[cosmwasm_schema::cw_serde]
pub struct UnbondingResponse {
    pub claims: Vec<Claim>,
}

#[cosmwasm_schema::cw_serde]
pub struct Claim {
    pub amount: Uint128,
    pub claimable_at: Expiration,
}
