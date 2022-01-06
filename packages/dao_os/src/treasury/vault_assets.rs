use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, Env, QueryRequest, StdError, StdResult, Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::query::terraswap::{query_asset_balance, query_pool};
use crate::tax::reverse_decimal;
use crate::treasury::msg::{ExternalValueResponse, ValueQueryMsg};
use crate::treasury::state::*;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::PoolResponse;

/// Every VaultAsset provides a way to determine its value recursivly relative to
/// a base asset.
/// This is subject to change as Chainlink an/or TWAP implementations roll out on terra.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct VaultAsset {
    pub asset: Asset,
    // The value reference provides the tooling to get the value of the holding
    // relative to the base asset.
    pub value_reference: Option<ValueRef>,
}

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ValueRef {
    /// A pool address of an asset/asset pair
    /// Both assets must be defined in the Vault_assets state
    Pool {
        pair_address: Addr,
    },
    // Liquidity pool addr for LP tokens
    Liquidity {
        pool_address: Addr,
    },
    // Or a Proxy, the proxy also takes a Decimal (the multiplier)
    // Asset will be valued as if they are Proxy tokens
    Proxy {
        proxy_asset: AssetInfo,
        multiplier: Decimal,
    },
    // Query an external contract to get the value
    External {
        contract_address: Addr,
    },
}

impl VaultAsset {
    /// Calculates the value of the asset through the optionally provided ValueReference
    pub fn value(
        &mut self,
        deps: Deps,
        env: &Env,
        set_holding: Option<Uint128>,
    ) -> StdResult<Uint128> {
        // Query how many of these tokens are held in the contract if not set.

        let holding: Uint128 = match set_holding {
            Some(setter) => setter,
            None => query_asset_balance(deps, &self.asset.info, env.contract.address.clone())?,
        };
        self.asset.amount = holding;

        // Is there a reference to calculate the value?
        if let Some(value_reference) = self.value_reference.as_ref() {
            match value_reference {
                // A Pool refers to a swap pair that recursively leads to an asset/base_asset pool.
                ValueRef::Pool { pair_address } => {
                    return self.asset_value(deps, env, pair_address)
                }
                // Liquidity is an LP token, value() fn is called recursively on both assets in the pool
                ValueRef::Liquidity { pool_address } => {
                    // Check if we have a Token
                    if let AssetInfo::Token { .. } = &self.asset.info {
                        return lp_value(deps, env, pool_address, &holding);
                    } else {
                        return Err(StdError::generic_err("Can't have a native LP token"));
                    }
                }
                // A proxy asset is used instead
                ValueRef::Proxy {
                    proxy_asset,
                    multiplier,
                } => return proxy_value(deps, env, proxy_asset, multiplier, holding),
                ValueRef::External { contract_address } => {
                    let response: ExternalValueResponse =
                        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                            contract_addr: contract_address.to_string(),
                            msg: to_binary(&ValueQueryMsg {
                                asset_info: self.asset.info.clone(),
                                amount: self.asset.amount,
                            })?,
                        }))?;
                    return Ok(response.value);
                }
            }
        }

        // If there is no valueref, it means this token is the base token.
        Ok(holding)
    }

    /// Calculates the value of an asset compared to some base asset throug the provided trading pair.
    pub fn asset_value(&self, deps: Deps, env: &Env, pool_addr: &Addr) -> StdResult<Uint128> {
        let pool_info: PoolResponse = query_pool(deps, pool_addr)?;
        // Get price
        let ratio = Decimal::from_ratio(pool_info.assets[0].amount, pool_info.assets[1].amount);

        let mut recursive_vault_asset: VaultAsset;
        let amount_in_other_denom: Uint128;
        // Get the value of the current asset in the denom of the other asset
        if self.asset.info == pool_info.assets[0].info {
            recursive_vault_asset =
                VAULT_ASSETS.load(deps.storage, get_identifier(&pool_info.assets[1].info))?;
            amount_in_other_denom = self.asset.amount * reverse_decimal(ratio);
        } else {
            recursive_vault_asset =
                VAULT_ASSETS.load(deps.storage, get_identifier(&pool_info.assets[0].info))?;
            amount_in_other_denom = self.asset.amount * ratio;
        }
        // Call value on this other asset.
        recursive_vault_asset.value(deps, env, Some(amount_in_other_denom))
    }
}

/// The proxy struct acts as an Asset overwrite.
/// By setting this proxy you define the asset to be some
/// other asset with a multiplier.
/// For example: AssetInfo = bluna, BaseAsset = uusd, Proxy: luna, multiplier = 1
/// Each bluna would be valued as one luna.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Proxy {
    // Proxy asset
    proxy_asset: AssetInfo,
    // Can be set to some constant or set to price,
    multiplier: Decimal,
}

impl Proxy {
    pub fn new(multiplier: Decimal, proxy_asset: AssetInfo) -> StdResult<Self> {
        Ok(Self {
            proxy_asset,
            multiplier,
        })
    }
}

/// Gets the identifier of the asset (either its denom or contract address)
pub fn get_identifier(asset_info: &AssetInfo) -> &String {
    match asset_info {
        AssetInfo::NativeToken { denom } => denom,
        AssetInfo::Token { contract_addr } => contract_addr,
    }
}
