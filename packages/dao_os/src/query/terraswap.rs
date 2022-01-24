use cosmwasm_std::{
    to_binary, Addr, Coin, Decimal, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};
use terraswap::asset::{Asset, AssetInfo};
use terraswap::pair::{PoolResponse, QueryMsg, SimulationResponse};
use terraswap::querier::{query_balance, query_token_balance};

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
pub fn query_pool(deps: Deps, pool_address: &Addr) -> StdResult<PoolResponse> {
    let response: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pool {})?,
    }))?;

    Ok(response)
}

pub fn pool_ratio(deps: Deps, pool_address: Addr) -> StdResult<Decimal> {
    let response: PoolResponse = deps.querier.query(&QueryRequest::Wasm(WasmQuery::Smart {
        contract_addr: pool_address.to_string(),
        msg: to_binary(&QueryMsg::Pool {})?,
    }))?;
    // [0,1]
    let ratio = Decimal::from_ratio(response.assets[0].amount, response.assets[1].amount);
    Ok(ratio)
}

pub fn query_asset_balance(
    deps: Deps,
    asset_info: &AssetInfo,
    address: Addr,
) -> StdResult<Uint128> {
    let amount = match asset_info.clone() {
        AssetInfo::NativeToken { denom } => query_balance(&deps.querier, address, denom)?,
        AssetInfo::Token { contract_addr } => query_token_balance(
            &deps.querier,
            deps.api.addr_validate(contract_addr.as_str())?,
            address,
        )?,
    };
    Ok(amount)
}
