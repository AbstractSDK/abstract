use cosmwasm_std::{to_binary, Binary, Deps, Env, Order, StdError};

use crate::contract::{GasStationApp, GasStationResult};
use crate::error::GasStationError;
use crate::msg::{
    GasPassHoldersResponse, GasPassResponse, GasStationQueryMsg, GradeInfoResponse,
    GradeListResponse,
};
use crate::state::{GasPumpItem, GradeName, GAS_PASSES, GRADES, GRADE_TO_USERS};

pub fn query_handler(
    deps: Deps,
    _env: Env,
    app: &GasStationApp,
    msg: GasStationQueryMsg,
) -> GasStationResult<Binary> {
    match msg {
        GasStationQueryMsg::GradeInfo { grade } => to_binary(&query_grade(deps, app, grade)?),
        GasStationQueryMsg::GradeList {} => to_binary(&query_grade_list(deps, app)?),
        GasStationQueryMsg::GasPassHolders { grade } => {
            to_binary(&query_gas_pass_holders(deps, app, grade)?)
        }
        GasStationQueryMsg::GasPass { holder } => to_binary(&query_gas_pass(deps, app, holder)?),
    }
    .map_err(Into::into)
}

fn query_gas_pass(
    deps: Deps,
    _app: &GasStationApp,
    holder: String,
) -> GasStationResult<GasPassResponse> {
    let holder = deps.api.addr_validate(&holder)?;
    let gas_pass = GAS_PASSES
        .load(deps.storage, &holder)
        .map_err(|_| GasStationError::HolderNotFound(holder.to_string()))?;

    Ok(GasPassResponse {
        grade: gas_pass.grade,
        expiration: gas_pass.expiration,
        holder: holder.to_string(),
    })
}

fn query_gas_pass_holders(
    deps: Deps,
    _app: &GasStationApp,
    grade: GradeName,
) -> GasStationResult<GasPassHoldersResponse> {
    let holders = GRADE_TO_USERS
        .load(deps.storage, &grade)
        .map_err(|_| GasStationError::GradeNotFound(grade.clone()))?;

    Ok(GasPassHoldersResponse {
        holders: holders.into_iter().collect(),
    })
}

/// Query info on a single pump
fn query_grade(
    deps: Deps,
    _app: &GasStationApp,
    grade: GradeName,
) -> GasStationResult<GradeInfoResponse> {
    let pump = GRADES
        .may_load(deps.storage, grade.clone())?
        .ok_or_else(|| GasStationError::GradeNotFound(grade.clone()))?;

    Ok(GradeInfoResponse {
        grade,
        fuel_mix: pump.fuel_mix,
    })
}

fn query_grade_list(deps: Deps, _app: &GasStationApp) -> GasStationResult<GradeListResponse> {
    let pumps: Result<Vec<GasPumpItem>, StdError> = GRADES
        .range(deps.storage, None, None, Order::Ascending)
        .collect();

    let pump_infos = pumps?
        .into_iter()
        .map(|(grade, pump)| GradeInfoResponse {
            grade,
            fuel_mix: pump.fuel_mix,
        })
        .collect();

    Ok(GradeListResponse { grades: pump_infos })
}
