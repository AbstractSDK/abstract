use cosmwasm_std::{
    to_binary, QueryMsg, Addr, Coin, Decimal, Deps, QueryRequest, StdResult, Uint128, WasmQuery,
};

use white_whale::ust_vault::msg::{VaultQueryMsg};

pub fn get_vault_value(deps: Deps) -> StdResult<Uint128> {
    let response: ValueResponse =
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