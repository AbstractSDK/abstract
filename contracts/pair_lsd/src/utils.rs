use cosmwasm_std::{Decimal, Decimal256, Deps, Env, StdResult, Storage, Uint128, Uint256, Uint64};
use itertools::Itertools;
use std::cmp::Ordering;

use wyndex::asset::{AssetInfoValidated, Decimal256Ext, DecimalAsset};
use wyndex::pair::TWAP_PRECISION;

use crate::math::{apply_rate, calc_y};
use crate::state::{get_precision, Config};
use wyndex::pair::ContractError;

/// Select offer and ask pools based on given offer and ask infos.
/// This function works with pools with up to 5 assets. Returns (offer_pool, ask_pool) in case of success.
/// If it is impossible to define offer and ask pools, returns [`ContractError`].
///
/// * **offer_asset_info** - asset info of the offer asset.
///
/// * **ask_asset_info** - asset info of the ask asset.
///
/// * **pools** - list of pools.
pub(crate) fn select_pools(
    offer_asset_info: Option<&AssetInfoValidated>,
    ask_asset_info: Option<&AssetInfoValidated>,
    pools: &[DecimalAsset],
) -> Result<(DecimalAsset, DecimalAsset), ContractError> {
    if pools.len() == 2 {
        match (offer_asset_info, ask_asset_info) {
            (Some(offer_asset_info), _) => {
                let (offer_ind, offer_pool) = pools
                    .iter()
                    .find_position(|pool| pool.info.eq(offer_asset_info))
                    .ok_or(ContractError::AssetMismatch {})?;
                Ok((offer_pool.clone(), pools[(offer_ind + 1) % 2].clone()))
            }
            (_, Some(ask_asset_info)) => {
                let (ask_ind, ask_pool) = pools
                    .iter()
                    .find_position(|pool| pool.info.eq(ask_asset_info))
                    .ok_or(ContractError::AssetMismatch {})?;
                Ok((pools[(ask_ind + 1) % 2].clone(), ask_pool.clone()))
            }
            _ => Err(ContractError::VariableAssetMissed {}), // Should always be unreachable
        }
    } else if let (Some(offer_asset_info), Some(ask_asset_info)) =
        (offer_asset_info, ask_asset_info)
    {
        if ask_asset_info.eq(offer_asset_info) {
            return Err(ContractError::SameAssets {});
        }

        let offer_pool = pools
            .iter()
            .find(|pool| pool.info.eq(offer_asset_info))
            .ok_or(ContractError::AssetMismatch {})?;
        let ask_pool = pools
            .iter()
            .find(|pool| pool.info.eq(ask_asset_info))
            .ok_or(ContractError::AssetMismatch {})?;

        Ok((offer_pool.clone(), ask_pool.clone()))
    } else {
        Err(ContractError::VariableAssetMissed {}) // Should always be unreachable
    }
}

/// Compute the current pool amplification coefficient (AMP).
pub(crate) fn compute_current_amp(config: &Config, env: &Env) -> StdResult<Uint64> {
    let block_time = env.block.time.seconds();
    if block_time < config.next_amp_time {
        let elapsed_time: Uint128 = block_time.saturating_sub(config.init_amp_time).into();
        let time_range = config
            .next_amp_time
            .saturating_sub(config.init_amp_time)
            .into();
        let init_amp = Uint128::from(config.init_amp);
        let next_amp = Uint128::from(config.next_amp);

        if next_amp > init_amp {
            let amp_range = next_amp - init_amp;
            let res = init_amp + (amp_range * elapsed_time).checked_div(time_range)?;
            Ok(res.try_into()?)
        } else {
            let amp_range = init_amp - next_amp;
            let res = init_amp - (amp_range * elapsed_time).checked_div(time_range)?;
            Ok(res.try_into()?)
        }
    } else {
        Ok(Uint64::from(config.next_amp))
    }
}

/// Returns a value using a newly specified precision.
///
/// * **value** value that will have its precision adjusted.
///
/// * **current_precision** `value`'s current precision
///
/// * **new_precision** new precision to use when returning the `value`.
pub(crate) fn adjust_precision(
    value: Uint128,
    current_precision: u8,
    new_precision: u8,
) -> StdResult<Uint128> {
    Ok(match current_precision.cmp(&new_precision) {
        Ordering::Equal => value,
        Ordering::Less => value.checked_mul(Uint128::new(
            10_u128.pow((new_precision - current_precision) as u32),
        ))?,
        Ordering::Greater => value.checked_div(Uint128::new(
            10_u128.pow((current_precision - new_precision) as u32),
        ))?,
    })
}

/// Structure for internal use which represents swap result.
pub(crate) struct SwapResult {
    pub return_amount: Uint128,
    pub spread_amount: Uint128,
}

/// Returns the result of a swap in form of a [`SwapResult`] object.
///
/// * **offer_asset** asset that is being offered.
///
/// * **offer_pool** pool of offered asset.
///
/// * **ask_pool** asked asset.
///
/// * **pools** array with assets available in the pool.
pub(crate) fn compute_swap(
    storage: &dyn Storage,
    env: &Env,
    config: &Config,
    offer_asset: &DecimalAsset,
    offer_pool: &DecimalAsset,
    ask_pool: &DecimalAsset,
    pools: &[DecimalAsset],
) -> Result<SwapResult, ContractError> {
    let token_precision = get_precision(storage, &ask_pool.info)?;

    let new_ask_pool = calc_y(
        offer_asset,
        &ask_pool.info,
        offer_pool.amount + offer_asset.amount,
        pools,
        compute_current_amp(config, env)?,
        token_precision,
        config,
    )?;

    let return_amount = ask_pool.amount.to_uint128_with_precision(token_precision)? - new_ask_pool;
    let offer_asset_amount = offer_asset
        .amount
        .to_uint128_with_precision(token_precision)?;

    // We consider swap rate to be target_rate in stable swap thus any difference is considered as spread.
    let spread_amount = apply_rate(&offer_asset.info, offer_asset_amount, config)
        .saturating_sub(apply_rate(&ask_pool.info, return_amount, config));

    Ok(SwapResult {
        return_amount,
        spread_amount,
    })
}

/// Accumulate token prices for the assets in the pool.
/// Returns the array of new prices for the asset combinations in the pool.
/// Empty if the config is still up to date.
///
/// *Important*: Make sure to update the target rate before calling this function.
///
/// * **pools** array with assets available in the pool *before* the operation.
pub fn accumulate_prices(
    deps: Deps,
    env: &Env,
    config: &mut Config,
    pools: &[DecimalAsset],
) -> Result<bool, ContractError> {
    let block_time = env.block.time.seconds();
    if block_time <= config.block_time_last {
        return Ok(false);
    }

    let time_elapsed = Uint128::from(block_time - config.block_time_last);

    if pools.iter().all(|pool| !pool.amount.is_zero()) {
        let immut_config = config.clone();
        for (from, to, value) in config.cumulative_prices.iter_mut() {
            let offer_asset = DecimalAsset {
                info: from.clone(),
                amount: Decimal256::one(),
            };

            let (offer_pool, ask_pool) = select_pools(Some(from), Some(to), pools)?;
            let SwapResult { return_amount, .. } = compute_swap(
                deps.storage,
                env,
                &immut_config,
                &offer_asset,
                &offer_pool,
                &ask_pool,
                pools,
            )?;

            *value = value.wrapping_add(time_elapsed.checked_mul(adjust_precision(
                return_amount,
                get_precision(deps.storage, &ask_pool.info)?,
                TWAP_PRECISION,
            )?)?);
        }
    }

    config.block_time_last = block_time;

    Ok(true)
}

/// Calculates the new price of B in terms of A, i.e. how many A you get for 1 B,
/// where A is the first asset in `config.pair_info.asset_infos` and B the second.
pub fn calc_new_price_a_per_b(
    deps: Deps,
    env: &Env,
    config: &Config,
    pools: &[DecimalAsset],
) -> Result<Decimal, ContractError> {
    calc_spot_price(
        deps,
        env,
        config,
        &config.pair_info.asset_infos[1],
        &config.pair_info.asset_infos[0],
        pools,
    )
}

pub fn calc_spot_price(
    deps: Deps,
    env: &Env,
    config: &Config,
    offer: &AssetInfoValidated,
    ask: &AssetInfoValidated,
    pools: &[DecimalAsset],
) -> Result<Decimal, ContractError> {
    let offer_asset = DecimalAsset {
        info: offer.clone(),
        // This is 1 unit (adjusted for number of decimals)
        amount: Decimal256::one(),
    };
    let (offer_pool, ask_pool) = select_pools(Some(offer), Some(ask), pools)?;

    // try swapping one unit to see how much we get
    let SwapResult { return_amount, .. } = compute_swap(
        deps.storage,
        env,
        config,
        &offer_asset,
        &offer_pool,
        &ask_pool,
        pools,
    )?;

    // Return amount is in number of base units. To make it decimal, we must divide by precision
    let decimals = get_precision(deps.storage, &ask_pool.info)?;
    let price = Decimal::from_atomics(return_amount, decimals as u32).unwrap();
    Ok(price)
}

#[allow(clippy::too_many_arguments)]
pub fn find_spot_price(
    deps: Deps,
    env: &Env,
    config: &Config,
    offer: AssetInfoValidated,
    ask: AssetInfoValidated,
    pools: Vec<DecimalAsset>,
    max_trade: Uint128,
    target_price: Decimal,
    iterations: u8,
) -> Result<Option<Uint128>, ContractError> {
    // normalize the max_trade with precision
    let decimals = get_precision(deps.storage, &offer)?;
    let mut trade = Decimal256::from_atomics(max_trade, decimals as u32).unwrap();

    // check min boundary (is price already too high)
    let current = calc_spot_price(deps, env, config, &offer, &ask, &pools)?;
    if current <= target_price {
        return Ok(None);
    }

    // check max boundary (if i swap all assets, is price still good enough)
    let max_pools = pools_after_swap(config, &offer, &ask, &pools, trade);
    let all_in = calc_spot_price(deps, env, config, &offer, &ask, &max_pools)?;

    // if this does not fit, recurse to find it (otherwise just use the max trade)
    if all_in < target_price {
        trade = recurse_bisect_spot_price(
            deps,
            env,
            config,
            &offer,
            &ask,
            &pools,
            Decimal256::zero(),
            trade,
            target_price,
            iterations,
        )?;
    }

    let amount = trade * Uint256::from(10_u128.pow(decimals as u32));
    Ok(Some(amount.try_into().unwrap()))
}

#[allow(clippy::too_many_arguments)]
pub fn recurse_bisect_spot_price(
    deps: Deps,
    env: &Env,
    config: &Config,
    offer: &AssetInfoValidated,
    ask: &AssetInfoValidated,
    pools: &[DecimalAsset],
    min_trade: Decimal256,
    max_trade: Decimal256,
    target_price: Decimal,
    iterations: u8,
) -> Result<Decimal256, ContractError> {
    // at the end, return mid-point
    let half = (min_trade + max_trade) / Decimal256::percent(200);
    if iterations == 0 {
        return Ok(half);
    }

    // find price at midpoint
    let mid_pools = pools_after_swap(config, offer, ask, pools, half);
    let mid_price = calc_spot_price(deps, env, config, offer, ask, &mid_pools)?;
    // and refine bounds up or down
    let (min_trade, max_trade) = match mid_price.cmp(&target_price) {
        std::cmp::Ordering::Equal => return Ok(half),
        std::cmp::Ordering::Greater => (half, max_trade),
        std::cmp::Ordering::Less => (min_trade, half),
    };

    // recurse with one less iteration
    recurse_bisect_spot_price(
        deps,
        env,
        config,
        offer,
        ask,
        pools,
        min_trade,
        max_trade,
        target_price,
        iterations - 1,
    )
}

/// Pretend we swapped amount from token into to token.
/// Return the pools value as if this happened to use for future calculations
fn pools_after_swap(
    config: &Config,
    offer: &AssetInfoValidated,
    ask: &AssetInfoValidated,
    pools: &[DecimalAsset],
    mut amount: Decimal256,
) -> Vec<DecimalAsset> {
    pools
        .iter()
        .cloned()
        .map(|mut asset| {
            if config.is_lsd(&asset.info) {
                amount /= Decimal256::from(config.target_rate());
            }
            if &asset.info == offer {
                asset.amount += amount;
                asset
            } else if &asset.info == ask {
                asset.amount -= amount;
                asset
            } else {
                asset
            }
        })
        .collect()
}
