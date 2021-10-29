use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr , Deps, StdResult};
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};

// We define a custom struct for each query response
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultInfo {
    pub asset_infos: Vec<AssetInfo>,
    pub contract_addr: Addr,
    pub liquidity_token: Addr,
}

impl VaultInfo {
    pub fn to_raw(&self, deps: Deps) -> StdResult<VaultInfoRaw> {
        let mut asset_infos: Vec<AssetInfoRaw> = vec![];
        for asset in &self.asset_infos {
            // iterate and push
            asset_infos.push(asset.to_raw(deps.api)?)
        }
        Ok(VaultInfoRaw {
            liquidity_token: deps.api.addr_canonicalize(&self.liquidity_token.as_str())?,
            contract_addr: deps.api.addr_canonicalize(&self.contract_addr.as_str())?,
            asset_infos,
        })
    }
}


#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultInfoRaw {
    pub asset_infos: Vec<AssetInfoRaw>,
    pub contract_addr: CanonicalAddr,
    pub liquidity_token: CanonicalAddr,
}

impl VaultInfoRaw {
    pub fn to_normal(&self, deps: Deps) -> StdResult<VaultInfo> {
        let mut asset_infos: Vec<AssetInfo> = vec![];
        for asset in &self.asset_infos {
            // iterate and push
            asset_infos.push(asset.to_normal(deps.api)?)
        }
        Ok(VaultInfo {
            liquidity_token: deps.api.addr_humanize(&self.liquidity_token.clone())?,
            contract_addr: deps.api.addr_humanize(&self.contract_addr.clone())?,
            asset_infos,
        })
    }

    pub fn query_pool(&self, deps: Deps, contract_addr: Addr) -> StdResult<Vec<Asset>> {
        let mut pool_assets : Vec<Asset> = vec![];

        for asset in &self.asset_infos {
            // iterate and push
            let info = asset.to_normal(deps.api)?;
            pool_assets.push(Asset {
                amount: info.query_pool(&deps.querier, deps.api, contract_addr.clone())?,
                info: info,
            });
        }
        Ok(pool_assets)
    }
}
