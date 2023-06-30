use abstract_adapter_utils::Identify;

pub const KUJIRA: &str = "kujira";

// Source https://docs.rs/kujira/0.8.2/kujira/
#[derive(Default)]
pub struct Kujira {}

impl Identify for Kujira {
    fn name(&self) -> &'static str {
        KUJIRA
    }
    fn is_available_on(&self, chain_name: &str) -> bool {
        chain_name == "kujira"
    }
}

#[cfg(feature = "kujira")]
use ::{
    abstract_core::objects::PoolAddress,
    abstract_dex_adapter_traits::{
        coins_in_assets, DexCommand, DexError, Fee, FeeOnInput, Return, Spread,
    },
    abstract_sdk::cw_helpers::wasm_smart_query,
    cosmwasm_std::{
        wasm_execute, Addr, Coin, CosmosMsg, Decimal, Decimal256, Deps, StdError, StdResult,
        Uint128,
    },
    cw_asset::{Asset, AssetInfo, AssetInfoBase},
    kujira::{
        bow::{
            self,
            market_maker::{ConfigResponse, PoolResponse},
        },
        fin,
    },
};

#[cfg(feature = "kujira")]
impl DexCommand for Kujira {
    fn swap(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
        belief_price: Option<Decimal>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let fin_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap, liquidity: _ } => swap,
            _ => panic!("invalid address"),
        };

        let swap_msg: Vec<CosmosMsg> = match &offer_asset.info {
            AssetInfo::Native(_) => vec![wasm_execute(
                fin_pair_address.to_string(),
                &fin::ExecuteMsg::Swap {
                    offer_asset: Some(Coin::try_from(&offer_asset)?),
                    belief_price: if let Some(belief_price) = belief_price {
                        Some(decimal2decimal256(belief_price)?)
                    } else {
                        None
                    },
                    max_spread: if let Some(max_spread) = max_spread {
                        Some(decimal2decimal256(max_spread)?)
                    } else {
                        None
                    },
                    to: None,
                    callback: None,
                },
                vec![offer_asset.clone().try_into()?],
            )?
            .into()],
            _ => panic!("unsupported asset"),
        };
        Ok(swap_msg)
    }

    fn provide_liquidity(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        mut offer_assets: Vec<Asset>,
        max_spread: Option<Decimal>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let fin_pair_address: Addr;
        let bow_pair_address: Addr;

        match pool_id {
            PoolAddress::SeparateAddresses { swap, liquidity } => {
                fin_pair_address = swap;
                bow_pair_address = liquidity;
            }
            _ => panic!("invalid address"),
        }
        let mut msgs = vec![];

        // We know that (+)two assets were provided because it's a requirement to resolve the pool
        // We don't know if one of the asset amounts is 0, which would require a simulation and swap before providing liquidity
        if offer_assets.len() > 2 {
            return Err(DexError::TooManyAssets(2));
        } else if offer_assets.iter().any(|a| a.amount.is_zero()) {
            // find 0 asset
            let (index, non_zero_offer_asset) = offer_assets
                .iter()
                .enumerate()
                .find(|(_, a)| !a.amount.is_zero())
                .ok_or(DexError::TooFewAssets {})?;

            // the other asset in offer_assets is the one with amount zero
            let ask_asset = offer_assets.get((index + 1) % 2).unwrap().info.clone();

            // we want to offer half of the non-zero asset to swap into the ask asset
            let offer_asset = Asset::new(
                non_zero_offer_asset.info.clone(),
                non_zero_offer_asset
                    .amount
                    .checked_div(Uint128::from(2u128))
                    .unwrap(),
            );

            // creating pool_id clones to avoid borrowing a partially moved value - didn't use .clone() on pool_id since
            // SeparateAddresses contains Addr values which doesn't have a Copy trait
            let pool_id_clone = PoolAddress::SeparateAddresses {
                swap: deps.api.addr_validate(fin_pair_address.as_ref())?,
                liquidity: deps.api.addr_validate(bow_pair_address.as_ref())?,
            };

            let pool_id_clone_2 = PoolAddress::SeparateAddresses {
                swap: deps.api.addr_validate(fin_pair_address.as_ref())?,
                liquidity: deps.api.addr_validate(bow_pair_address.as_ref())?,
            };

            // simulate swap to get the amount of ask asset we can provide after swapping
            let simulated_received = self
                .simulate_swap(deps, pool_id_clone, offer_asset.clone(), ask_asset.clone())?
                .0;
            let swap_msg = self.swap(
                deps,
                pool_id_clone_2,
                offer_asset.clone(),
                ask_asset.clone(),
                None,
                max_spread,
            )?;
            // add swap msg
            msgs.extend(swap_msg);
            // update the offer assets for providing liquidity
            offer_assets = vec![offer_asset, Asset::new(ask_asset, simulated_received)];
        }

        // execute msg
        let msg = bow::market_maker::ExecuteMsg::Deposit {
            max_slippage: max_spread,
            callback: None,
        };

        let coins = coins_in_assets(&offer_assets);

        // actual call to pair
        let liquidity_msg = wasm_execute(bow_pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn provide_liquidity_symmetric(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        paired_assets: Vec<AssetInfo>,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        // unimplemented!();
        let bow_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap: _, liquidity } => liquidity,
            _ => panic!("invalid address"),
        };
        let mut msgs = vec![];

        if paired_assets.len() > 1 {
            return Err(DexError::TooManyAssets(2));
        }

        // Pair config
        let pair_config: ConfigResponse = deps.querier.query(&wasm_smart_query(
            bow_pair_address.to_string(),
            &bow::market_maker::QueryMsg::Config {},
        )?)?;

        // Get pair info
        let pair_info: PoolResponse = deps.querier.query(&wasm_smart_query(
            bow_pair_address.to_string(),
            &bow::market_maker::QueryMsg::Pool {},
        )?)?;

        let pair_assets: Vec<kujira::Asset> = vec![
            kujira::Asset {
                amount: pair_info.balances[0],
                info: kujira::AssetInfo::NativeToken {
                    denom: pair_config.denoms[0].clone(),
                },
            },
            kujira::Asset {
                amount: pair_info.balances[1],
                info: kujira::AssetInfo::NativeToken {
                    denom: pair_config.denoms[1].clone(),
                },
            },
        ];
        let kujira_offer_asset = cw_asset_to_kujira(&offer_asset)?;
        let other_asset = if pair_assets[0].info == kujira_offer_asset.info {
            let price = Decimal::from_ratio(pair_assets[1].amount, pair_assets[0].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else if pair_assets[1].info == kujira_offer_asset.info {
            let price = Decimal::from_ratio(pair_assets[0].amount, pair_assets[1].amount);
            let other_token_amount = price * offer_asset.amount;
            Asset {
                amount: other_token_amount,
                info: paired_assets[0].clone(),
            }
        } else {
            return Err(DexError::ArgumentMismatch(
                offer_asset.to_string(),
                pair_config.denoms.iter().map(|e| e.to_string()).collect(),
            ));
        };

        let offer_assets = [offer_asset, other_asset];

        let coins = coins_in_assets(&offer_assets);

        let msg = bow::market_maker::ExecuteMsg::Deposit {
            max_slippage: None,
            callback: None,
        };

        // actual call to pair
        let liquidity_msg = wasm_execute(bow_pair_address, &msg, coins)?.into();
        msgs.push(liquidity_msg);

        Ok(msgs)
    }

    fn withdraw_liquidity(
        &self,
        _deps: Deps,
        pool_id: PoolAddress,
        lp_token: Asset,
    ) -> Result<Vec<CosmosMsg>, DexError> {
        let bow_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap: _, liquidity } => liquidity,
            _ => panic!("invalid address"),
        };

        // execute msg
        let msg = bow::market_maker::ExecuteMsg::Withdraw { callback: None };
        let funds = vec![Coin::try_from(lp_token)?];
        let withdraw_msg = wasm_execute(bow_pair_address, &msg, funds)?.into();
        Ok(vec![withdraw_msg])
    }

    fn simulate_swap(
        &self,
        deps: Deps,
        pool_id: PoolAddress,
        offer_asset: Asset,
        _ask_asset: AssetInfo,
    ) -> Result<(Return, Spread, Fee, FeeOnInput), DexError> {
        let fin_pair_address: Addr = match pool_id {
            PoolAddress::SeparateAddresses { swap, liquidity: _ } => swap,
            _ => panic!("invalid address"),
        };
        // Do simulation
        let fin::SimulationResponse {
            return_amount,
            spread_amount,
            commission_amount,
        } = deps.querier.query(&wasm_smart_query(
            fin_pair_address.to_string(),
            &fin::QueryMsg::Simulation {
                offer_asset: cw_asset_to_kujira(&offer_asset)?,
            },
        )?)?;
        // commission paid in result asset
        Ok((
            Uint128::try_from(return_amount).unwrap(),
            Uint128::try_from(spread_amount).unwrap(),
            Uint128::try_from(commission_amount).unwrap(),
            false,
        ))
    }
}

#[cfg(feature = "kujira")]
fn cw_asset_to_kujira(asset: &Asset) -> Result<kujira::Asset, DexError> {
    match &asset.info {
        AssetInfoBase::Native(denom) => Ok(kujira::Asset {
            amount: asset.amount,
            info: kujira::AssetInfo::NativeToken {
                denom: denom.into(),
            },
        }),
        _ => Err(DexError::UnsupportedAssetType(asset.info.to_string())),
    }
}

#[cfg(feature = "kujira")]
/// Converts [`Decimal`] to [`Decimal256`].
pub fn decimal2decimal256(dec_value: Decimal) -> StdResult<Decimal256> {
    Decimal256::from_atomics(dec_value.atomics(), dec_value.decimal_places()).map_err(|_| {
        StdError::generic_err(format!(
            "Failed to convert Decimal {} to Decimal256",
            dec_value
        ))
    })
}
