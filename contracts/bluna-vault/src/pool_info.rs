use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Deps, StdResult};
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfo {
    pub asset_infos: [AssetInfo; 2],
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
    pub slippage: Decimal,
    // pub total_user_deposits: Uint128
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct PoolInfoRaw {
    pub asset_infos: [AssetInfoRaw; 2],
    pub contract_addr: CanonicalAddr,
    pub liquidity_token: CanonicalAddr,
    pub slippage: Decimal,
}

impl PoolInfoRaw {
    pub fn to_normal(&self, deps: Deps) -> StdResult<PoolInfo> {
        Ok(PoolInfo {
            liquidity_token: deps.api.addr_humanize(&self.liquidity_token.clone())?,
            contract_addr: deps.api.addr_humanize(&self.contract_addr.clone())?,
            slippage: self.slippage,
            asset_infos: [
                self.asset_infos[0].to_normal(deps.api)?,
                self.asset_infos[1].to_normal(deps.api)?,
            ],
        })
    }

    pub fn query_pools(&self, deps: Deps, contract_addr: Addr) -> StdResult<[Asset; 2]> {
        let info_0: AssetInfo = self.asset_infos[0].to_normal(deps.api)?;
        let info_1: AssetInfo = self.asset_infos[1].to_normal(deps.api)?;
        Ok([
            Asset {
                amount: info_0.query_pool(&deps.querier, deps.api, contract_addr.clone())?,
                info: info_0,
            },
            Asset {
                amount: info_1.query_pool(&deps.querier, deps.api, contract_addr)?,
                info: info_1,
            },
        ])
    }
}
