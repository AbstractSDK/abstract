use abstract_os::api::{ApiExecuteMsg, ApiInterfaceMsg};
use cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Empty, StdResult};
use serde::Serialize;

pub fn api_req<T: Serialize>(
    api_address: impl Into<String>,
    message: impl Into<ApiInterfaceMsg<T>>,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let api_msg: ApiInterfaceMsg<T> = message.into();
    Ok(wasm_execute(api_address, &api_msg, funds)?.into())
}

pub fn configure_api(
    api_address: impl Into<String>,
    message: ApiExecuteMsg,
) -> StdResult<CosmosMsg> {
    let api_msg: ApiInterfaceMsg<Empty> = message.into();
    Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
}
