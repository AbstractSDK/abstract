use crate::ContractError;
use crate::{state::SUDO_PARAMS, SudoMsg, SudoParams};
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{DepsMut, Env, Event, Response};
// use sg_std::Response;

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn sudo(deps: DepsMut, _env: Env, msg: SudoMsg) -> Result<Response, ContractError> {
    match msg {
        SudoMsg::UpdateParams { max_record_count } => sudo_update_params(deps, max_record_count),
    }
}

pub fn sudo_update_params(deps: DepsMut, max_record_count: u32) -> Result<Response, ContractError> {
    SUDO_PARAMS.save(deps.storage, &SudoParams { max_record_count })?;

    let event =
        Event::new("update-params").add_attribute("max_record_count", max_record_count.to_string());
    Ok(Response::new().add_event(event))
}
