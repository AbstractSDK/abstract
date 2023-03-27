//! # Proxy Asset
//! Proxy assets are objects that describe an asset and a way to calculate that asset's value against a base asset.
//!
//! ## Details
//! A proxy asset is composed of two components.
//! * The `asset`, which is an [`AssetInfo`].
//! * The [`PriceSource`] which is an enum that indicates how to calculate the value for that asset.
//!
//! The base asset is the asset for which `price_source` in `None`.
//! **There should only be ONE base asset when configuring your proxy**

use super::{
    ans_host::AnsHost, asset_entry::AssetEntry, DexAssetPairing, LpToken, PoolAddress,
    PoolReference,
};
use crate::{error::AbstractError, AbstractResult};
use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, QuerierWrapper, QueryRequest, StdError, Uint128, WasmQuery,
};
use cw_asset::{Asset, AssetInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// represents the conversion of an asset in terms of the provided asset
/// Example: provided asset is ETH and the price source for ETH is the pair ETH/USD, the price is 100USD/ETH
/// then `AssetConversion { into: USD, ratio: 100}`
pub struct AssetConversion {
    into: AssetInfo,
    ratio: Decimal,
}

impl AssetConversion {
    pub fn new(asset: impl Into<AssetInfo>, price: Decimal) -> Self {
        Self {
            into: asset.into(),
            ratio: price,
        }
    }
    /// convert the balance of an asset into a (list of) asset(s) given the provided rate(s)
    pub fn convert(rates: &[Self], amount: Uint128) -> Vec<Asset> {
        rates
            .iter()
            .map(|rate| Asset::new(rate.into.clone(), amount * rate.ratio))
            .collect()
    }
}

/// Provides information on how to calculate the value of an asset
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[non_exhaustive]
pub enum UncheckedPriceSource {
    /// A pool address of an asset/asset pair
    /// Both assets must be defined in the Proxy_assets state
    Pair(DexAssetPairing),
    // Liquidity Pool token
    LiquidityToken {},
    // a Proxy, the proxy also takes a Decimal (the multiplier)
    // Asset will be valued as if they are Proxy tokens
    ValueAs {
        asset: AssetEntry,
        multiplier: Decimal,
    },
    None,
}

impl UncheckedPriceSource {
    pub fn check(
        self,
        deps: Deps,
        ans_host: &AnsHost,
        entry: &AssetEntry,
    ) -> AbstractResult<PriceSource> {
        match self {
            UncheckedPriceSource::Pair(pair_info) => {
                let PoolReference {
                    pool_address,
                    unique_id,
                } = ans_host
                    .query_asset_pairing(&deps.querier, &pair_info)?
                    .pop()
                    .unwrap();
                let pool_assets = ans_host
                    .query_pool_metadata(&deps.querier, &unique_id)?
                    .assets;
                let assets = ans_host.query_assets(&deps.querier, &pool_assets)?;
                // TODO: fix this for pools with multiple assets
                assert_eq!(assets.len(), 2);
                // TODO: fix this for Osmosis pools
                pool_address.expect_contract()?;
                Ok(PriceSource::Pool {
                    address: pool_address,
                    pair: assets,
                })
            }
            UncheckedPriceSource::LiquidityToken {} => {
                let lp_token = LpToken::try_from(entry.clone())?;
                let token_entry: AssetEntry = lp_token.clone().into();
                let pairing = DexAssetPairing::try_from(token_entry)?;
                let pool_assets = ans_host.query_assets(&deps.querier, &lp_token.assets)?;
                // TODO: fix this for multiple pools with same pair
                let pool_address = ans_host
                    .query_asset_pairing(&deps.querier, &pairing)?
                    .pop()
                    .unwrap()
                    .pool_address;
                Ok(PriceSource::LiquidityToken {
                    pool_assets,
                    pool_address,
                })
            }
            UncheckedPriceSource::ValueAs { asset, multiplier } => {
                let asset_info = ans_host.query_asset(&deps.querier, &asset)?;
                Ok(PriceSource::ValueAs {
                    asset: asset_info,
                    multiplier,
                })
            }
            UncheckedPriceSource::None => Ok(PriceSource::None),
        }
    }
}

/// Provides information on how to calculate the value of an asset
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[non_exhaustive]
pub enum PriceSource {
    /// Should only be used for the base asset
    None,
    /// A pool name of an asset/asset pair
    /// Both assets must be defined in the Vault_assets state
    Pool {
        address: PoolAddress,
        /// two assets that make up a pair in the pool
        pair: Vec<AssetInfo>,
    },
    /// Liquidity pool token
    LiquidityToken {
        pool_assets: Vec<AssetInfo>,
        pool_address: PoolAddress,
    },
    /// Asset will be valued as if they are ValueAs.asset tokens
    ValueAs {
        asset: AssetInfo,
        multiplier: Decimal,
    },
}

impl PriceSource {
    /// Returns the assets that are required to calculate the price of the asset
    /// Panics if the price source is None
    pub fn dependencies(&self, asset: &AssetInfo) -> Vec<AssetInfo> {
        match self {
            // return the other asset as the dependency
            PriceSource::Pool { pair, .. } => {
                pair.iter().filter(|a| *a != asset).cloned().collect()
            }
            PriceSource::LiquidityToken { pool_assets, .. } => pool_assets.clone(),
            PriceSource::ValueAs { asset, .. } => vec![asset.clone()],
            PriceSource::None => vec![],
        }
    }

    /// Calculates the conversion ratio of the asset.
    pub fn conversion_rates(
        &self,
        deps: Deps,
        asset: &AssetInfo,
    ) -> AbstractResult<Vec<AssetConversion>> {
        // Is there a reference to calculate the price?
        // each method must return the price of the asset in terms of the another asset, accept for the base asset.
        match self {
            // A Pool refers to a swap pair, the ratio of assets in the pool represents the price of the asset in the other asset's denom
            PriceSource::Pool { address, pair } => self
                .trade_pair_price(deps, asset, &address.expect_contract()?, pair)
                .map(|e| vec![e]),
            // Liquidity is an LP token,
            PriceSource::LiquidityToken {
                pool_address,
                pool_assets,
            } => self.lp_conversion(deps, asset, &pool_address.expect_contract()?, pool_assets),
            // A proxy asset is used instead
            PriceSource::ValueAs { asset, multiplier } => {
                Ok(vec![AssetConversion::new(asset.clone(), *multiplier)])
            }
            // None means it's the base asset
            PriceSource::None => Ok(vec![]),
        }
    }

    /// Calculates the price of an asset compared to some other asset through the provided trading pair.
    fn trade_pair_price(
        &self,
        deps: Deps,
        priced_asset: &AssetInfo,
        address: &Addr,
        pair: &[AssetInfo],
    ) -> AbstractResult<AssetConversion> {
        let other_asset_info = pair.iter().find(|a| a != &priced_asset).unwrap();
        // query assets held in pool, gives price
        let pool_info = (
            other_asset_info.query_balance(&deps.querier, address)?,
            priced_asset.query_balance(&deps.querier, address)?,
        );
        // other / this
        let ratio = Decimal::from_ratio(pool_info.0.u128(), pool_info.1.u128());
        // Get the conversion ratio in the denom of this asset
        // #other = #this * (pool_other/pool_this)
        Ok(AssetConversion::new(other_asset_info.clone(), ratio))
    }

    /// Calculate the conversions of an LP token
    /// Uses the lp token name to query pair pool for both assets
    /// Returns the conversion ratio of the LP token in terms of the other asset
    fn lp_conversion(
        &self,
        deps: Deps,
        lp_asset: &AssetInfo,
        pool: &Addr,
        pool_assets: &[AssetInfo],
    ) -> AbstractResult<Vec<AssetConversion>> {
        let supply: Uint128;
        if let AssetInfo::Cw20(addr) = lp_asset {
            supply = query_cw20_supply(&deps.querier, addr)?;
        } else {
            return Err(StdError::generic_err("Can't have a native LP token").into());
        }
        pool_assets
            .iter()
            .map(|a| {
                let balance = a
                    .query_balance(&deps.querier, pool.clone())
                    .map_err(AbstractError::from)?;
                Ok(AssetConversion::new(
                    a.clone(),
                    Decimal::from_ratio(balance.u128(), supply.u128()),
                ))
            })
            .collect::<AbstractResult<Vec<AssetConversion>>>()
    }
}

fn query_cw20_supply(querier: &QuerierWrapper, contract_addr: &Addr) -> AbstractResult<Uint128> {
    let response: cw20::TokenInfoResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.into(),
            msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {})?,
        }))?;
    Ok(response.total_supply)
}
