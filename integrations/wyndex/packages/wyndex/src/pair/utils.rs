use std::str::FromStr;

use super::error::ContractError;

use crate::asset::{Asset, AssetInfo, AssetInfoValidated, AssetValidated};

use cosmwasm_std::{
    from_slice, wasm_execute, Addr, Api, CosmosMsg, Decimal, Fraction, QuerierWrapper, StdError,
    StdResult, Uint128,
};
use cw20::Cw20ExecuteMsg;
use itertools::Itertools;

/// The default swap slippage
pub const DEFAULT_SLIPPAGE: &str = "0.005";
/// The maximum allowed swap slippage
pub const MAX_ALLOWED_SLIPPAGE: &str = "0.5";

/// This function makes raw query to the factory contract and
/// checks whether the pair needs to update an owner or not.
pub fn migration_check(
    querier: QuerierWrapper,
    factory: &Addr,
    pair_addr: &Addr,
) -> StdResult<bool> {
    if let Some(res) = querier.query_wasm_raw(factory, b"pairs_to_migrate".as_slice())? {
        let res: Vec<Addr> = from_slice(&res)?;
        Ok(res.contains(pair_addr))
    } else {
        Ok(false)
    }
}

/// Helper function to check if the given asset infos are valid.
pub fn check_asset_infos(
    api: &dyn Api,
    asset_infos: &[AssetInfo],
) -> Result<Vec<AssetInfoValidated>, ContractError> {
    if !asset_infos.iter().all_unique() {
        return Err(ContractError::DoublingAssets {});
    }

    asset_infos
        .iter()
        .map(|asset_info| asset_info.validate(api))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Helper function to check that the assets in a given array are valid.
pub fn check_assets(api: &dyn Api, assets: &[Asset]) -> Result<Vec<AssetValidated>, ContractError> {
    if !assets.iter().map(|a| a.info.clone()).all_unique() {
        return Err(ContractError::DoublingAssets {});
    }

    assets
        .iter()
        .map(|asset| asset.validate(api))
        .collect::<Result<Vec<_>, _>>()
        .map_err(Into::into)
}

/// Checks that cw20 token is part of the pool.
///
/// * **cw20_sender** is cw20 token address which is being checked.
pub fn check_cw20_in_pool(
    asset_infos: &[AssetInfoValidated],
    cw20_sender: &Addr,
) -> Result<(), ContractError> {
    for asset_info in asset_infos {
        match asset_info {
            AssetInfoValidated::Token(contract_addr) if contract_addr == cw20_sender => {
                return Ok(())
            }
            _ => {}
        }
    }

    Err(ContractError::Unauthorized {})
}

/// If `belief_price` and `max_spread` are both specified, we compute a new spread,
/// otherwise we just use the swap spread to check `max_spread`.
///
/// * **belief_price** belief price used in the swap.
///
/// * **max_spread** max spread allowed so that the swap can be executed successfully.
///
/// * **offer_amount** amount of assets to swap.
///
/// * **return_amount** amount of assets to receive from the swap.
///
/// * **spread_amount** spread used in the swap.
pub fn assert_max_spread(
    belief_price: Option<Decimal>,
    max_spread: Option<Decimal>,
    offer_amount: Uint128,
    return_amount: Uint128,
    spread_amount: Uint128,
) -> Result<(), ContractError> {
    let default_spread = Decimal::from_str(DEFAULT_SLIPPAGE)?;
    let max_allowed_spread = Decimal::from_str(MAX_ALLOWED_SLIPPAGE)?;

    let max_spread = max_spread.unwrap_or(default_spread);
    if max_spread.gt(&max_allowed_spread) {
        return Err(ContractError::AllowedSpreadAssertion {});
    }

    if let Some(belief_price) = belief_price {
        let expected_return = offer_amount
            * belief_price.inv().ok_or_else(|| {
                ContractError::Std(StdError::generic_err(
                    "Invalid belief_price. Check the input values.",
                ))
            })?;

        let spread_amount = expected_return.saturating_sub(return_amount);

        if return_amount < expected_return
            && Decimal::from_ratio(spread_amount, expected_return) > max_spread
        {
            return Err(ContractError::MaxSpreadAssertion {});
        }
    } else if Decimal::from_ratio(spread_amount, return_amount + spread_amount) > max_spread {
        return Err(ContractError::MaxSpreadAssertion {});
    }

    Ok(())
}

/// Mint LP tokens for a beneficiary
///
/// * **recipient** LP token recipient.
///
/// * **amount** amount of LP tokens that will be minted for the recipient.
///
pub fn mint_token_message(
    token: &Addr,
    recipient: &Addr,
    amount: Uint128,
) -> Result<Vec<CosmosMsg>, ContractError> {
    Ok(vec![wasm_execute(
        token,
        &Cw20ExecuteMsg::Mint {
            recipient: recipient.to_string(),
            amount,
        },
        vec![],
    )?
    .into()])
}

/// Return the amount of tokens that a specific amount of LP tokens would withdraw.
///
/// * **pools** array with assets available in the pool.
///
/// * **amount** amount of LP tokens to calculate underlying amounts for.
///
/// * **total_share** total amount of LP tokens currently issued by the pool.
pub fn get_share_in_assets(
    pools: &[AssetValidated],
    amount: Uint128,
    total_share: Uint128,
) -> Vec<AssetValidated> {
    let mut share_ratio = Decimal::zero();
    if !total_share.is_zero() {
        share_ratio = Decimal::from_ratio(amount, total_share);
    }

    pools
        .iter()
        .map(|pool| AssetValidated {
            info: pool.info.clone(),
            amount: pool.amount * share_ratio,
        })
        .collect()
}
