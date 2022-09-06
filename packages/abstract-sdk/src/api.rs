use abstract_os::api::{BaseExecuteMsg, ExecuteMsg};
use cosmwasm_std::{wasm_execute, Coin, CosmosMsg, Empty, StdResult};
use serde::Serialize;

pub fn api_req<T: Serialize>(
    api_address: impl Into<String>,
    message: impl Into<ExecuteMsg<T>>,
    funds: Vec<Coin>,
) -> StdResult<CosmosMsg> {
    let api_msg: ExecuteMsg<T> = message.into();
    Ok(wasm_execute(api_address, &api_msg, funds)?.into())
}

pub fn configure_api(
    api_address: impl Into<String>,
    message: BaseExecuteMsg,
) -> StdResult<CosmosMsg> {
    let api_msg: ExecuteMsg<Empty> = message.into();
    Ok(wasm_execute(api_address, &api_msg, vec![])?.into())
}
