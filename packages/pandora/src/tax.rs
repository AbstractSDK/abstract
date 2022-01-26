use cosmwasm_std::{
    to_binary, Addr, BankMsg, Coin, CosmosMsg, Decimal, Deps, Fraction, StdResult, Uint128, WasmMsg,
};
use cw20::Cw20ExecuteMsg;
use terra_cosmwasm::TerraQuerier;
use terraswap::asset::{Asset, AssetInfo};

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

pub fn into_msg_without_tax(asset: Asset, recipient: Addr) -> StdResult<CosmosMsg> {
    let amount = asset.amount;

    match &asset.info {
        AssetInfo::Token { contract_addr } => Ok(CosmosMsg::Wasm(WasmMsg::Execute {
            contract_addr: contract_addr.to_string(),
            msg: to_binary(&Cw20ExecuteMsg::Transfer {
                recipient: recipient.to_string(),
                amount,
            })?,
            funds: vec![],
        })),
        AssetInfo::NativeToken { denom } => Ok(CosmosMsg::Bank(BankMsg::Send {
            to_address: recipient.to_string(),
            amount: vec![Coin::new(asset.amount.u128(), denom)],
        })),
    }
}
