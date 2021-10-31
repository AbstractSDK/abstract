use cosmwasm_std::{
    to_binary, Addr, Coin, Decimal, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};
use terraswap::asset::{Asset, AssetInfo, PairInfo};
use terraswap::pair::{PoolResponse, QueryMsg, SimulationResponse};

pub fn simulate_swap(deps: Deps, pool_address: Addr, offer_coin: Coin) -> StdResult<Uint128> {
    let response: SimulationResponse =
        deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
            contract_addr: pool_address.to_string(),
            msg: to_binary(&QueryMsg::Simulation {
                offer_asset: Asset {
                    info: AssetInfo::NativeToken {
                        denom: offer_coin.denom,
                    },
                    amount: offer_coin.amount,
                },
            })?,
        }))?;

    Ok(response.return_amount)
}

// perform a query for Pool information using the provided pool_address
// return any response.
// PoolResponse comes from terraswap and contains info on each of the assets as well as total share
pub fn query_pool(deps: Deps, pool_address: Addr) -> StdResult<PoolResponse> {
    let response: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pool {})?,
    }))?;

    Ok(response)
}

// perform a query for the LP Token Pair information using the provided pool_address
// return only the address. TODO: Review if we should return the full response instead
pub fn query_lp_token(deps: Deps, pool_address: Addr) -> StdResult<String> {
    let response: PairInfo = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pair {})?,
    }))?;

    Ok(response.liquidity_token)
}

pub fn pool_ratio(deps: Deps, pool_address: Addr) -> StdResult<Decimal> {
    let response: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pool {})?,
    }))?;
    // [ust,luna]
    let ratio = Decimal::from_ratio(response.assets[0].amount, response.assets[1].amount);
    Ok(ratio)
}
