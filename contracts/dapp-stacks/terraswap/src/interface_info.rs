use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr , Deps, StdResult};
use terraswap::asset::{Asset, AssetInfo, AssetInfoRaw};

/// IntefaceInfo struct hold all the addresses needed for this strategy.
/// New strategies will require other interfaces. 
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InterfaceInfo {
    pub pool_address: Addr,
    pub bluna_hub_address: Addr,
}

impl InterfaceInfo {
    pub fn to_raw(&self, deps: Deps) -> StdResult<InterfaceInfoRaw> {
        Ok(InterfaceInfoRaw {
            pool_address: deps.api.addr_canonicalize(&self.pool_address.as_str())?,
            bluna_hub_address: deps.api.addr_canonicalize(&self.bluna_hub_address.as_str())?,
        })
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InterfaceInfoRaw {
    pub bluna_hub_address: CanonicalAddr,
    pub pool_address: CanonicalAddr,
}

impl InterfaceInfoRaw {
    pub fn to_normal(&self, deps: Deps) -> StdResult<InterfaceInfo> {
        Ok(InterfaceInfo {
            pool_address: deps.api.addr_humanize(&self.pool_address.clone())?,
            bluna_hub_address: deps.api.addr_humanize(&self.bluna_hub_address.clone())?,
        })
    }
}
