use cosmwasm_std::{to_binary, Binary, Deps, Env};

use crate::contract::{AppResult, GasStationApp};
use crate::error::AppError;
use crate::msg::{GasPumpInfoResponse, GasStationQueryMsg};
use crate::state::GAS_PUMPS;

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &GasStationApp,
    msg: GasStationQueryMsg,
) -> AppResult<Binary> {
    match msg {
        GasStationQueryMsg::GasPumpInfo { grade } => to_binary(&query_pump(deps, app, grade)?),
    }
    .map_err(Into::into)
}
/// Get dca
fn query_pump(deps: Deps, _app: &GasStationApp, grade: String) -> AppResult<GasPumpInfoResponse> {
    let pump = GAS_PUMPS
        .may_load(deps.storage, grade.clone())?
        .ok_or_else(|| AppError::GasPumpNotfound(grade.clone()))?;

    Ok(GasPumpInfoResponse {
        grade,
        fuel_mix: pump.fuel_mix,
        denom: pump.denom,
    })
}
