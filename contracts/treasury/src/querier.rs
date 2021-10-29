use crate::state::LUNA_DENOM;
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{QueryMsg as PairQueryMsg, SimulationResponse};

use cosmwasm_std::{to_binary, Coin, Decimal, Deps, QueryRequest, StdResult, Uint128, WasmQuery};
use terra_cosmwasm::TerraQuerier;

pub fn from_micro(amount: Uint128) -> Decimal {
    Decimal::from_ratio(amount, Uint128::from(1000000u64))
}

pub fn query_luna_price_on_terraswap(
    deps: Deps,
    pool_address: String,
    amount: Uint128,
) -> StdResult<Uint128> {
    let response: SimulationResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_address,
            msg: to_binary(&PairQueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: LUNA_DENOM.to_string(),
                    },
                    amount,
                },
            })?,
        }))?;

    Ok(response.return_amount)
}

pub fn query_market_price(deps: Deps, offer_coin: Coin, ask_denom: String) -> StdResult<Uint128> {
    let querier = TerraQuerier::new(&deps.querier);
    let response = querier.query_swap(offer_coin, ask_denom)?;
    Ok(response.receive.amount)
}
