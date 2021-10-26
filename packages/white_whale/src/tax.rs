use cosmwasm_std::{Coin, Decimal, Deps, Fraction, StdResult, Uint128};
use terra_cosmwasm::TerraQuerier;

pub fn deduct_tax(deps: Deps, coin: Coin) -> StdResult<Coin> {
    let tax_amount = compute_tax(deps, &coin)?;
    Ok(Coin {
        denom: coin.denom,
        amount: coin.amount - tax_amount,
    })
}

pub fn compute_tax(deps: Deps, coin: &Coin) -> StdResult<Uint128> {
    let terra_querier = TerraQuerier::new(&deps.querier);
    let tax_rate = (terra_querier.query_tax_rate()?).rate;
    let tax_cap = (terra_querier.query_tax_cap(coin.denom.to_string())?).cap;
    let amount = coin.amount;
    Ok(std::cmp::min(
        amount - amount * reverse_decimal(Decimal::one() + tax_rate),
        tax_cap,
    ))
}

pub fn reverse_decimal(decimal: Decimal) -> Decimal {
    decimal.inv().unwrap_or_default()
}
