use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdError};

use crate::contract::{GasStationResult, GasStationApp};
use crate::error::GasStationError;
use crate::msg::{GasPumpInfoResponse, GasPumpListResponse, GasStationQueryMsg};
use crate::state::{GAS_PUMPS, GasPumpItem};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &GasStationApp,
    msg: GasStationQueryMsg,
) -> GasStationResult<Binary> {
    match msg {
        GasStationQueryMsg::GasPumpInfo { grade } => to_binary(&query_pump(deps, app, grade)?),
        GasStationQueryMsg::GasPumpList { } => to_binary(&query_pump_list(deps, app)?),
    }
    .map_err(Into::into)
}
/// Query info on a single pump
fn query_pump(deps: Deps, _app: &GasStationApp, grade: String) -> GasStationResult<GasPumpInfoResponse> {
    let pump = GAS_PUMPS
        .may_load(deps.storage, grade.clone())?
        .ok_or_else(|| GasStationError::GasPumpNotfound(grade.clone()))?;

    Ok(GasPumpInfoResponse {
        grade,
        fuel_mix: pump.fuel_mix,
        denom: pump.denom,
    })
}

fn query_pump_list(deps: Deps, _app: &GasStationApp) -> GasStationResult<GasPumpListResponse> {
    let pumps: Result<Vec<GasPumpItem>, StdError> = GAS_PUMPS.range(deps.storage, None, None, Order::Ascending)
        .collect();

    let pump_infos = pumps?
        .into_iter()
        .map(|(grade, pump)| GasPumpInfoResponse {
            grade,
            fuel_mix: pump.fuel_mix,
            denom: pump.denom,
        })
        .collect();

    Ok(GasPumpListResponse {
        pumps: pump_infos,
    })
}
