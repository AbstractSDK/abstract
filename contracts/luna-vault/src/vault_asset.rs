use cosmwasm_std::{StdError,Uint128,Addr, Decimal, StdResult, Deps, CosmosMsg};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use terraswap::asset::{AssetInfo,Asset};
use terraswap::pair::{PoolResponse};
use terraswap::querier::query_supply;
use crate::state::VAULT_ASSETS;
use white_whale::tax::reverse_decimal;
use white_whale::query::terraswap::{query_pool, pool_ratio};
// Example/contracts/mocks/mock_terraswap/terraswap_pair/src/contract.rs

/// Every VaultAsset provides a way to determine its value relative to either
/// the base asset or equivalent to a certain amount of some other asset,
/// which in its turn can be decomposed into some base asset value.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct VaultAsset<'a>{
    pub asset: Asset,
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

impl <'a> VaultAsset <'a> {

    pub fn value(&self, deps: Deps) -> StdResult<Uint128> {
        
        // Query how many of these tokens I hold. 
        //let holdings = self.asset.info.query_pool(&deps.querier, deps.api, owner_addr)?;
        let holdings = self.asset.amount;
        
        // Is there a reference to calculate the value? 
        if let Some(value_reference) = self.value_reference.as_ref() {
            
            match value_reference {
                ValueRef::Pool { address } => {
                    return self.asset_value(deps, &address)
                },
                ValueRef::Liquidity { pool_address, proxy } => {
                    // Check if we have a Token
                    if let AssetInfo::Token {contract_addr}  =  self.asset.info {
                        
                        return lp_value(deps, pool_address, proxy, holdings);
                    } else {
                        return Err(StdError::generic_err("Can't have a native LP token"));
                    }
                },
                ValueRef::Proxy {proxy_asset, multiplier, proxy_pool} => {
                    // TODO
                }
            }
            
        } else {
            // No ValueRef so this should be the base token. 
            // TODO: Add error in case this is not true.
            /*if base_asset != self.asset.info {
                return Err(StdError::generic_err("No value conversion provided for this asset."));
            }*/
            return Ok(holdings);
        }
        
        Ok(Uint128::zero())
    }

    pub fn asset_value(&self, deps: Deps, pool_addr: &Addr)-> StdResult<Uint128>{
        let pool_info: PoolResponse = query_pool(deps, pool_addr.clone())?;
        let ratio = Decimal::from_ratio(pool_info.assets[0].amount, pool_info.assets[1].amount);
        if self.asset.info == pool_info.assets[0].info {
            return Ok(self.asset.amount * reverse_decimal(ratio))
        } else {
            return Ok(self.asset.amount * ratio)
        }
    }

}

pub fn lp_value(deps: Deps, pool_addr: &Addr, proxy: &Proxy, holdings: Uint128) -> StdResult<Uint128> {
    // Get LP pool info
    let pool_info: PoolResponse = query_pool(deps, pool_addr.clone())?;
    
    // Set total supply of LP tokens
    let total_lp = pool_info.total_share;
    let share = holdings/total_lp;

    let asset_1 = pool_info.assets[0];
    let asset_2 = pool_info.assets[1];
     
    // load the assets
    let mut vault_asset_1: VaultAsset = VAULT_ASSETS.load(deps.storage, asset_1.info)?;
    let mut vault_asset_2: VaultAsset = VAULT_ASSETS.load(deps.storage, asset_2.info)?;

    // set the amounts to the LP holdings
    vault_asset_1.asset.amount = share * asset_1.amount;
    vault_asset_2.asset.amount = share * asset_2.amount;

    // Call value on these assets. 
    Ok(vault_asset_1.value(deps)? + vault_asset_2.value(deps)?)
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



// #[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
// pub struct Liquidity <'a> {
//     pool_address: Addr,
//     #[serde(borrow)]
//     proxy: Proxy<'a>,
// }

// impl Liquidity <'_> {
//     pub fn value(&self, deps: Deps, holdings: Uint128, base_asset: AssetInfo) -> StdResult<Uint128> {
//         // Get total in pool
//         // Calculate share
//         // Use proxy to get value

//     }
// }
 