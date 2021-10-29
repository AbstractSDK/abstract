use cosmwasm_std::{StdError,Uint128,Addr, Decimal, StdResult, Deps, CosmosMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::AssetInfo;


/// Every VaultAsset provides a way to determine its value relative to either
/// the base asset or equivalent to a certain amount of some other asset,
/// which in its turn can be decomposed into some base asset value.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultAsset<'a>{
    pub asset_info: AssetInfo,
    #[serde(borrow)]
    // The value reference provides the tooling to get the value of the holding 
    // relative to the base asset. 
    pub value_reference: Option<ValueRef<'a>>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub enum ValueRef <'a> {
    // A pool address of the asset/base_asset pair
    Pool{
        address: Addr,
    },
    // Liquidity pool addr to get fraction of owned liquidity
    // proxy to calculate value of both assets held by liquidity
    Liquidity{
        pool_address: Addr,
        proxy: Proxy<'a>,
    },
    // Or a Proxy, the proxy also takes a Decimal (the multiplier)
    // Asset will be valued as if they are Proxy tokens
    Proxy{
        proxy_asset: &'a str,
        multiplier: Decimal,
        proxy_pool: Option<Addr>,
    },
}

impl <'a> VaultAsset <'a>{
    pub fn value(&self, deps: Deps, owner_addr: Addr, base_asset: AssetInfo) -> StdResult<Uint128> {
        // Query how many of these tokens I hold. 
        let holdings = self.asset_info.query_pool(&deps.querier, deps.api, owner_addr)?;

        // Is there a reference to calculate the value? 
        if let Some(value_reference) = self.value_reference {
            match value_reference {
                ValueRef::Pool { address } => {
                    // TODO
                    return Ok(Uint128::zero());
                },
                ValueRef::Liquidity { pool_address, proxy } => {
                    
                },
                ValueRef::Proxy {proxy_asset, multiplier, proxy_pool} => {
                    
                }
            }
        } else {
            // No ValueRef so this should be the base token. 
            // TODO: Add error in case this is not true.
            if base_asset != self.asset_info {
                return Err(StdError::generic_err("No value conversion provided for this asset."));
            }
            return Ok(holdings);
        }
        
        Ok(Uint128::zero())
    }
}
/// The proxy struct acts as an Asset overwrite.
/// By setting this proxy you define the asset to be some 
/// other asset while also providing the relevant pool 
/// address for that asset.
/// For example: AssetInfo = bluna, BaseAsset = uusd, Proxy: Luna/ust pool
/// proxy_pool = bluna/luna, multiplier = proxy_pool bluna price
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Proxy <'a> {
    // Proxy asset, str is used for querying in asset_map.
    proxy_asset: &'a str,
    // Can be set to some constant or set to price,
    multiplier: Decimal,
    // LP pool to get multiplier
    proxy_pool: Option<Addr>,
}

impl <'a> Proxy <'a> {
    pub fn new(asset_name: &'a str, multiplier: Decimal, proxy_pool: Option<Addr>) -> Self {
        Self {
            proxy_asset: asset_name,
            multiplier,
            proxy_pool,
        }
    }
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Liquidity <'a> {
    pool_address: Addr,
    #[serde(borrow)]
    proxy: Proxy<'a>,
}

impl Liquidity <'_> {
    pub fn value(&self, deps: Deps, holdings: Uint128, base_asset: AssetInfo) -> StdResult<Uint128> {
        // Get total in pool
        // Calculate share
        // Use proxy to get value

    }
}
 