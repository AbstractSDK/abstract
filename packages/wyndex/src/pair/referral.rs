use crate::{
    asset::{AssetInfoExt, AssetValidated},
    factory::ConfigResponse,
    querier::query_factory_config,
};

use cosmwasm_std::{Addr, CosmosMsg, Decimal, Decimal256, QuerierWrapper, Uint128, Uint256};

use super::ContractError;

/// Deducts the referral commission from the given offer asset and
/// adds the send message it to the given `messages`.
///
/// This errors if the referral commission is greater than the maximum or
/// the factory cannot be queried.
pub fn handle_referral(
    factory_config: &ConfigResponse,
    referral_address: Option<Addr>,
    referral_commission: Option<Decimal>,
    offer_asset: &mut AssetValidated,
    messages: &mut Vec<CosmosMsg>,
) -> Result<(), ContractError> {
    if let Some(referral_address) = referral_address {
        let commission_amount = take_referral(factory_config, referral_commission, offer_asset)?;

        // send commission_amount to referral_address
        if !commission_amount.is_zero() {
            let commission = offer_asset.info.with_balance(commission_amount);
            messages.push(commission.into_msg(referral_address)?);
        }
    }

    Ok(())
}

/// Subtracts the amount of tokens that should be sent to the referral from the given asset
/// and returns the subtracted amount.
///
/// This errors if the referral commission is greater than the maximum or
/// the factory cannot be queried.
pub fn take_referral(
    factory_config: &ConfigResponse,
    referral_commission: Option<Decimal>,
    offer_asset: &mut AssetValidated,
) -> Result<Uint128, ContractError> {
    // no need to load factory config if there is no referral commission
    if referral_commission == Some(Decimal::zero()) {
        return Ok(Uint128::zero());
    }

    let referral_commission = referral_commission.unwrap_or(factory_config.max_referral_commission);

    // error if referral commission is too high
    if referral_commission > factory_config.max_referral_commission {
        return Err(ContractError::ReferralCommissionTooHigh {});
    }

    // subtract commission_amount from offer_asset
    let commission_amount = offer_asset.amount * referral_commission;
    offer_asset.amount -= commission_amount;

    Ok(commission_amount)
}

/// Given an offer asset, this function adds the referral commission to it,
/// such that applying [`take_referral`] to the result will return the original offer asset.
/// It also returns the commission amount as a second return value.
pub fn add_referral(
    querier: &QuerierWrapper,
    factory_addr: &Addr,
    referral: bool,
    referral_commission: Option<Decimal>,
    mut offer_asset: AssetValidated,
) -> Result<(AssetValidated, Uint128), ContractError> {
    // no need to load factory config if there is no referral commission
    if !referral || referral_commission == Some(Decimal::zero()) {
        return Ok((offer_asset, Uint128::zero()));
    }

    let factory_config = query_factory_config(querier, factory_addr.to_string())?;
    let referral_commission = referral_commission.unwrap_or(factory_config.max_referral_commission);

    // error if referral commission is too high
    if referral_commission > factory_config.max_referral_commission {
        return Err(ContractError::ReferralCommissionTooHigh {});
    }

    // calculate commission_amount
    // The basic formula is: `(offer_asset.amount + commission_amount) * referral_commission = commission_amount`.
    // We can transform that to: `commission_amount = offer_asset.amount * referral_commission / (1 - referral_commission)`.
    // We use Decimal256 to avoid the overflow panic on big `offer_asset.amount`.
    let referral_commission: Decimal256 = referral_commission.into();
    let commission_amount = Decimal256::from_ratio(offer_asset.amount, 1u128) * referral_commission
        / (Decimal256::one() - referral_commission);
    // We can safely convert back to Uint128, because the commission amount is always less than the offer asset amount.
    let commission_amount: Uint128 = (commission_amount * Uint256::one())
        .try_into()
        .expect("commission_amount should fit into Uint128");
    // subtract commission_amount from offer_asset
    offer_asset.amount += commission_amount;

    Ok((offer_asset, commission_amount))
}
