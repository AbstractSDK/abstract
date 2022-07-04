use cosmwasm_std::{Addr, QuerierWrapper, StdError};

use cosmwasm_std::StdResult;

use crate::core::common::query_os_id;

use super::state::{Core, OS_ADDRESSES};

pub fn verify_os_manager(
    querier: &QuerierWrapper,
    maybe_manager: &Addr,
    version_control_addr: &Addr,
) -> StdResult<Core> {
    let os_id = query_os_id(querier, maybe_manager)?;
    let maybe_os = OS_ADDRESSES.query(querier, version_control_addr.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => {
            if &core.manager != maybe_manager {
                Err(StdError::generic_err(
                    "Proposed manager is not the manager of this instance.",
                ))
            } else {
                Ok(core)
            }
        }
    }
}

pub fn verify_os_proxy(
    querier: &QuerierWrapper,
    maybe_proxy: &Addr,
    version_control_addr: &Addr,
) -> StdResult<Core> {
    let os_id = query_os_id(querier, maybe_proxy)?;
    let maybe_os = OS_ADDRESSES.query(querier, version_control_addr.clone(), os_id)?;
    match maybe_os {
        None => Err(StdError::generic_err(format!(
            "OS with id {} is not active.",
            os_id
        ))),
        Some(core) => {
            if &core.proxy != maybe_proxy {
                Err(StdError::generic_err(
                    "Proposed proxy is not the proxy of this instance.",
                ))
            } else {
                Ok(core)
            }
        }
    }
}
