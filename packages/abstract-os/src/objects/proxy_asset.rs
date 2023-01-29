//! # Proxy Asset
//! Proxy assets are objects that describe an asset and a way to calculate that asset's value against a base asset.
//!
//! ## Details
//! A proxy asset is composed of two components.
//! * The `asset`, which is an [`AssetEntry`] and maps to an [`AssetInfo`].
//! * The [`ValueRef`] which is an enum that indicates how to calculate the value for that asset.
//!
//! The base asset is the asset for which `value_reference` in `None`.
//! **There should only be ONE base asset when configuring your proxy**

use super::{
    ans_host::AnsHost,
    asset_entry::AssetEntry,
    contract_entry::{ContractEntry, UncheckedContractEntry},
};
use crate::{
    manager::state::OS_MODULES,
    proxy::{
        state::{ADMIN, VAULT_ASSETS},
        ExternalValueResponse, ValueQueryMsg,
    },
};
use cosmwasm_std::{
    to_binary, Addr, Decimal, Deps, Env, QuerierWrapper, QueryRequest, StdError, StdResult,
    Uint128, WasmQuery,
};
use cw_asset::{Asset, AssetInfo};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::convert::TryInto;

/// A proxy asset with unchecked ans_host entry fields.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct UncheckedProxyAsset {
    /// The asset that's held by the proxy
    pub asset: String,
    /// The value reference provides the tooling to get the value of the asset
    /// relative to the base asset.
    /// If None, the provided asset is set as the base asset.
    /// **You can only have one base asset!**
    pub value_reference: Option<UncheckedValueRef>,
}

impl UncheckedProxyAsset {
    pub fn new(asset: impl Into<String>, value_reference: Option<UncheckedValueRef>) -> Self {
        Self {
            asset: asset.into(),
            value_reference,
        }
    }

    /// Perform checks on the proxy asset to ensure it can be resolved by the AnsHost
    pub fn check(self, deps: Deps, ans_host: &AnsHost) -> StdResult<ProxyAsset> {
        let entry: AssetEntry = self.asset.into();
        ans_host.query_asset(&deps.querier, &entry)?;
        let value_reference = self
            .value_reference
            .map(|val| val.check(deps, ans_host, &entry));
        Ok(ProxyAsset {
            asset: entry,
            value_reference: value_reference.transpose()?,
        })
    }
}

/// Provides information on how to calculate the value of an asset
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]

pub enum UncheckedValueRef {
    /// A pool address of an asset/asset pair
    /// Both assets must be defined in the Proxy_assets state
    Pool {
        pair: String,
        exchange: String,
    },
    // Liquidity Pool token
    LiquidityToken {},
    // a Proxy, the proxy also takes a Decimal (the multiplier)
    // Asset will be valued as if they are Proxy tokens
    ValueAs {
        asset: String,
        multiplier: Decimal,
    },
    // Query an external contract to get the value
    External {
        api_name: String,
    },
}

impl UncheckedValueRef {
    pub fn check(self, deps: Deps, ans_host: &AnsHost, entry: &AssetEntry) -> StdResult<ValueRef> {
        match self {
            UncheckedValueRef::Pool { pair, exchange } => {
                let lowercase = pair.to_ascii_lowercase();
                let mut composite: Vec<&str> = lowercase.split('_').collect();
                if composite.len() != 2 {
                    return Err(StdError::generic_err(
                        "trading pair should be formatted as \"asset1_asset2\".",
                    ));
                }
                composite.sort();
                let pair_name = format!("{}_{}", composite[0], composite[1]);
                // verify pair is available
                let pair_contract: ContractEntry =
                    UncheckedContractEntry::new(exchange, pair_name).check();
                ans_host.query_contract(&deps.querier, &pair_contract)?;
                Ok(ValueRef::Pool {
                    pair: pair_contract,
                })
            }
            UncheckedValueRef::LiquidityToken {} => {
                let maybe_pair: UncheckedContractEntry = entry.to_string().try_into()?;
                // Ensure lp pair is registered
                ans_host.query_contract(&deps.querier, &maybe_pair.check())?;
                Ok(ValueRef::LiquidityToken {})
            }
            UncheckedValueRef::ValueAs { asset, multiplier } => {
                let replacement_asset: AssetEntry = asset.into();
                ans_host.query_asset(&deps.querier, &replacement_asset)?;
                Ok(ValueRef::ValueAs {
                    asset: replacement_asset,
                    multiplier,
                })
            }
            UncheckedValueRef::External { api_name } => Ok(ValueRef::External { api_name }),
        }
    }
}

/// Every ProxyAsset provides a way to determine its value recursively relative to
/// a base asset.
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ProxyAsset {
    /// Asset entry that maps to an AssetInfo using raw-queries on ans_host
    pub asset: AssetEntry,
    /// The value reference provides the tooling to get the value of the asset
    /// relative to the base asset.
    pub value_reference: Option<ValueRef>,
}

/// Provides information on how to calculate the value of an asset
#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, JsonSchema)]

pub enum ValueRef {
    /// A pool name of an asset/asset pair
    /// Both assets must be defined in the Vault_assets state
    Pool { pair: ContractEntry },
    /// Liquidity pool token
    LiquidityToken {},
    /// Asset will be valued as if they are ValueAs.asset tokens
    ValueAs {
        asset: AssetEntry,
        multiplier: Decimal,
    },
    /// Query an external contract to get the value
    External { api_name: String },
}

impl ProxyAsset {
    /// Calculates the value of the asset through the optionally provided ValueReference
    // TODO: improve efficiency
    // We could cache each asset/contract address and store each asset in a stack with the most complex (most hops) assets on top.
    // Doing this would prevent an asset value from being calculated multiple times.
    pub fn value(
        &mut self,
        deps: Deps,
        env: &Env,
        ans_host: &AnsHost,
        set_holding: Option<Uint128>,
    ) -> StdResult<Uint128> {
        // Query how many of these tokens are held in the contract if not set.
        let asset_info = ans_host.query_asset(&deps.querier, &self.asset)?;
        let holding: Uint128 = match set_holding {
            Some(setter) => setter,
            None => asset_info.query_balance(&deps.querier, env.contract.address.clone())?,
        };

        let valued_asset = Asset::new(asset_info, holding);

        // Is there a reference to calculate the value?
        if let Some(value_reference) = self.value_reference.clone() {
            match value_reference {
                // A Pool refers to a swap pair that recursively leads to an asset/base_asset pool.
                ValueRef::Pool { pair } => {
                    return self.trade_pair_value(deps, env, ans_host, valued_asset, pair)
                }
                // Liquidity is an LP token, value() fn is called recursively on both assets in the pool
                ValueRef::LiquidityToken {} => {
                    // We map the LP token to its pair address.
                    // lp tokens are stored as "dex/asset1_asset2" in the asset store.
                    // pairs are stored as ContractEntry{protocol: dex, contract: asset1_asset2} in the contract store.
                    let maybe_pair: UncheckedContractEntry = self.asset.to_string().try_into()?;
                    let pair = maybe_pair.check();
                    return self.lp_value(deps, env, ans_host, valued_asset, pair);
                }
                // A proxy asset is used instead
                ValueRef::ValueAs { asset, multiplier } => {
                    return value_as_value(deps, env, ans_host, asset, multiplier, holding)
                }
                ValueRef::External { api_name } => {
                    let manager = ADMIN.get(deps)?.unwrap();
                    let maybe_api_addr = OS_MODULES.query(&deps.querier, manager, &api_name)?;
                    if let Some(api_addr) = maybe_api_addr {
                        let response: ExternalValueResponse =
                            deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
                                contract_addr: api_addr.to_string(),
                                msg: to_binary(&ValueQueryMsg {
                                    asset: self.asset.clone(),
                                    amount: valued_asset.amount,
                                })?,
                            }))?;
                        return Ok(response.value);
                    } else {
                        return Err(StdError::generic_err(format!(
                            "external contract api {api_name} must be enabled on OS"
                        )));
                    }
                }
            }
        }
        // If there is no valueref, it means this token is the base token.
        Ok(holding)
    }

    /// Calculates the value of an asset compared to some base asset through the provided trading pair.
    pub fn trade_pair_value(
        &self,
        deps: Deps,
        env: &Env,
        ans_host: &AnsHost,
        valued_asset: Asset,
        pair: ContractEntry,
    ) -> StdResult<Uint128> {
        let other_pool_asset: AssetEntry =
            other_asset_name(self.asset.as_str(), &pair.contract)?.into();

        let pair_address = ans_host.query_contract(&deps.querier, &pair)?;
        let other_asset_info = ans_host.query_asset(&deps.querier, &other_pool_asset)?;

        // query assets held in pool, gives price
        let pool_info = (
            other_asset_info.query_balance(&deps.querier, &pair_address)?,
            valued_asset
                .info
                .query_balance(&deps.querier, pair_address)?,
        );

        // other / this
        let ratio = Decimal::from_ratio(pool_info.0.u128(), pool_info.1.u128());

        // Get the value of the current asset in the denom of the other asset
        let mut recursive_vault_asset = VAULT_ASSETS.load(deps.storage, other_pool_asset)?;

        // #other = #this * (pool_other/pool_this)
        let amount_in_other_denom = valued_asset.amount * ratio;
        // Call value on this other asset.
        recursive_vault_asset.value(deps, env, ans_host, Some(amount_in_other_denom))
    }

    /// Calculate the value of an LP token
    /// Uses the lp token name to query pair pool for both assets
    pub fn lp_value(
        &self,
        deps: Deps,
        env: &Env,
        ans_host: &AnsHost,
        lp_asset: Asset,
        pair: ContractEntry,
    ) -> StdResult<Uint128> {
        let supply: Uint128;
        if let AssetInfo::Cw20(addr) = &lp_asset.info {
            supply = query_cw20_supply(&deps.querier, addr)?;
        } else {
            return Err(StdError::generic_err("Can't have a native LP token"));
        }

        // Get total supply of LP tokens and calculate share
        let share: Decimal = Decimal::from_ratio(lp_asset.amount, supply.u128());

        let other_pool_asset_names = get_pair_asset_names(pair.contract.as_str());

        if other_pool_asset_names.len() != 2 {
            return Err(StdError::generic_err(format!(
                "lp pair contract {pair} must be composed of two assets."
            )));
        }

        let pair_address = ans_host.query_contract(&deps.querier, &pair)?;

        let asset_1 = ans_host.query_asset(&deps.querier, &other_pool_asset_names[0].into())?;
        let asset_2 = ans_host.query_asset(&deps.querier, &other_pool_asset_names[1].into())?;
        // query assets held in pool, gives price
        let (amount1, amount2) = (
            asset_1.query_balance(&deps.querier, &pair_address)?,
            asset_2.query_balance(&deps.querier, pair_address)?,
        );

        // load the assets
        let mut vault_asset_1: ProxyAsset =
            VAULT_ASSETS.load(deps.storage, other_pool_asset_names[0].into())?;
        let mut vault_asset_2: ProxyAsset =
            VAULT_ASSETS.load(deps.storage, other_pool_asset_names[1].into())?;

        // set the amounts to the LP holdings
        let vault_asset_1_amount = share * Uint128::new(amount1.u128());
        let vault_asset_2_amount = share * Uint128::new(amount2.u128());
        // Call value on these assets.
        Ok(
            vault_asset_1.value(deps, env, ans_host, Some(vault_asset_1_amount))?
                + vault_asset_2.value(deps, env, ans_host, Some(vault_asset_2_amount))?,
        )
    }
}

pub fn value_as_value(
    deps: Deps,
    env: &Env,
    ans_host: &AnsHost,
    replacement_asset: AssetEntry,
    multiplier: Decimal,
    holding: Uint128,
) -> StdResult<Uint128> {
    // Get the proxy asset
    let mut replacement_vault_asset: ProxyAsset =
        VAULT_ASSETS.load(deps.storage, replacement_asset)?;
    // call value on proxy asset with adjusted multiplier.
    replacement_vault_asset.value(deps, env, ans_host, Some(holding * multiplier))
}
/// Get the other asset's name from a composite name
/// ex: asset= "btc" composite = "btc_eth"
/// returns "eth"
pub fn other_asset_name<'a>(asset: &'a str, composite: &'a str) -> StdResult<&'a str> {
    composite
        .split('_')
        .find(|component| *component != asset)
        .ok_or_else(|| {
            StdError::generic_err(format!(
                "composite {composite} is not structured correctly"
            ))
        })
}

/// Composite of form asset1_asset2
pub fn get_pair_asset_names(composite: &str) -> Vec<&str> {
    composite.split('_').collect()
}

fn query_cw20_supply(querier: &QuerierWrapper, contract_addr: &Addr) -> StdResult<Uint128> {
    let response: cw20::TokenInfoResponse =
        querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: contract_addr.into(),
            msg: to_binary(&cw20::Cw20QueryMsg::TokenInfo {})?,
        }))?;
    Ok(response.total_supply)
}
