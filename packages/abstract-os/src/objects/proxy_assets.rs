use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, Env, QueryRequest, StdError, StdResult, Uint128, WasmQuery,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cw_asset::{Asset, AssetInfo};

use crate::proxy::{ExternalValueResponse, ValueQueryMsg};

/// Every ProxyAsset provides a way to determine its value recursivly relative to
/// a base asset.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ProxyAsset {
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
    Pool { pair_address: Addr },
    /// Liquidity pool addr for LP tokens
    Liquidity { pool_address: Addr },
    /// Or a Proxy, the proxy also takes a Decimal (the multiplier)
    /// Asset will be valued as if they are Proxy tokens
    Proxy {
        proxy_asset: AssetInfo,
        multiplier: Decimal,
    },
    /// Query an external contract to get the value
    External { contract_address: Addr },
}

impl ProxyAsset {
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
            None => self
                .asset
                .info
                .query_balance(&deps.querier, env.contract.address.clone())?,
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
                    if let AssetInfo::Cw20(..) = &self.asset.info {
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
                    todo!();
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
        todo!();
        // let pool_info: PoolResponse = query_pool(deps, pool_addr)?;
        // Get price
        // let ratio = Decimal::from_ratio(
        //     pool_info.assets[0].amount.u128(),
        //     pool_info.assets[1].amount.u128(),
        // );

        // let mut recursive_vault_asset: ProxyAsset;
        // let amount_in_other_denom: Uint128;
        // // Get the value of the current asset in the denom of the other asset
        // if cw_to_terraswap(&self.asset.info) == pool_info.assets[0].info {
        //     recursive_vault_asset = VAULT_ASSETS.load(
        //         deps.storage,
        //         get_tswap_asset_identifier(&pool_info.assets[1].info),
        //     )?;
        //     amount_in_other_denom = self.asset.amount * reverse_decimal(ratio);
        // } else {
        //     recursive_vault_asset = VAULT_ASSETS.load(
        //         deps.storage,
        //         get_tswap_asset_identifier(&pool_info.assets[0].info),
        //     )?;
        //     amount_in_other_denom = self.asset.amount * ratio;
        // }
        // // Call value on this other asset.
        // recursive_vault_asset.value(deps, env, Some(amount_in_other_denom))
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
// pub fn get_tswap_asset_identifier(asset_info: &terraswap::asset::AssetInfo) -> &String {
//     match asset_info {
//         terraswap::asset::AssetInfo::NativeToken { denom } => denom,
//         terraswap::asset::AssetInfo::Token { contract_addr } => contract_addr,
//     }
// }

/// Gets the identifier of the asset (either its denom or contract address)
pub fn get_asset_identifier(asset_info: &AssetInfo) -> String {
    match asset_info {
        AssetInfo::Native(denom) => denom.to_owned(),
        AssetInfo::Cw20(contract_addr) => contract_addr.into(),
    }
}

pub fn lp_value(deps: Deps, env: &Env, pool_addr: &Addr, holdings: &Uint128) -> StdResult<Uint128> {
    todo!();
    // Get LP pool info
    // let pool_info: PoolResponse = query_pool(deps, pool_addr)?;

    // // Get total supply of LP tokens and calculate share
    // let total_lp = pool_info.total_share;
    // let share: Decimal = Decimal::from_ratio(*holdings, total_lp.u128());

    // let asset_1 = &pool_info.assets[0];
    // let asset_2 = &pool_info.assets[1];

    // // load the assets
    // let mut vault_asset_1: ProxyAsset = VAULT_ASSETS.load(
    //     deps.storage,
    //     get_tswap_asset_identifier(&asset_1.info).as_str(),
    // )?;
    // let mut vault_asset_2: ProxyAsset = VAULT_ASSETS.load(
    //     deps.storage,
    //     get_tswap_asset_identifier(&asset_2.info).as_str(),
    // )?;

    // // set the amounts to the LP holdings
    // let vault_asset_1_amount = share * Uint128::new(asset_1.amount.u128());
    // let vault_asset_2_amount = share * Uint128::new(asset_2.amount.u128());
    // // Call value on these assets.
    // Ok(vault_asset_1.value(deps, env, Some(vault_asset_1_amount))?
    //     + vault_asset_2.value(deps, env, Some(vault_asset_2_amount))?)
}

pub fn proxy_value(
    deps: Deps,
    env: &Env,
    proxy_asset_info: &AssetInfo,
    multiplier: &Decimal,
    holding: Uint128,
) -> StdResult<Uint128> {
    todo!();
    // Get the proxy asset
    // let mut proxy_vault_asset: ProxyAsset = VAULT_ASSETS.load(
    //     deps.storage,
    //     get_asset_identifier(proxy_asset_info).as_str(),
    // )?;

    // // call value on proxy asset with adjusted multiplier.
    // proxy_vault_asset.value(deps, env, Some(holding * *multiplier))
}
