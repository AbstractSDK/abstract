use cosmwasm_std::{
    to_binary, Binary, Decimal, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
};
use cw_storage_plus::Item;

use crate::msg::{TargetQuery, TargetValueResponse};

const MOCK_RATE: Item<Decimal> = Item::new("mock_rate");

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    target_rate: Decimal,
) -> StdResult<Response> {
    MOCK_RATE.save(deps.storage, &target_rate)?;

    Ok(Response::new())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    new_rate: Decimal,
) -> StdResult<Response> {
    MOCK_RATE.save(deps.storage, &new_rate)?;

    Ok(Response::new())
}

pub fn query(deps: Deps, _env: Env, msg: TargetQuery) -> StdResult<Binary> {
    match msg {
        TargetQuery::TargetValue {} => {
            let target_value = MOCK_RATE.load(deps.storage)?;
            to_binary(&TargetValueResponse { target_value })
        }
    }
}
