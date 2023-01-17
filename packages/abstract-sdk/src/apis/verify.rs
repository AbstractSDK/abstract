//! # Verification
//! The `Verify` struct provides helper functions that enable the contract to verify if the sender is an OS, OS admin, etc.
use super::RegisterAccess;
use abstract_os::{
    manager::state::OS_ID,
    version_control::{state::OS_ADDRESSES, Core},
};
use cosmwasm_std::StdResult;
use cosmwasm_std::{Addr, Deps, StdError};

/// Verify if a sender's address is associated with an OS.
pub trait Verification: RegisterAccess {
    fn os_register<'a>(&'a self, deps: Deps<'a>) -> OsRegister<Self> {
        OsRegister { base: self, deps }
    }
}

impl<T> Verification for T where T: RegisterAccess {}

/// Endpoint for OS address verification
#[derive(Clone)]
pub struct OsRegister<'a, T: Verification> {
    base: &'a T,
    deps: Deps<'a>,
}

impl<'a, T: Verification> OsRegister<'a, T> {
    /// Verify if the provided manager address is indeed a user.
    pub fn assert_manager(&self, maybe_manager: &Addr) -> StdResult<Core> {
        let os_id = OS_ID
            .query(&self.deps.querier, maybe_manager.clone())
            .map_err(|_| StdError::generic_err("Caller must be an OS manager."))?;
        let maybe_os =
            OS_ADDRESSES.query(&self.deps.querier, self.base.registry(self.deps)?, os_id)?;
        match maybe_os {
            None => Err(StdError::generic_err(format!(
                "OS with id {} is not active.",
                os_id
            ))),
            Some(core) => {
                if &core.manager != maybe_manager {
                    Err(StdError::generic_err(
                        "Proposed manager is not the manager of this OS.",
                    ))
                } else {
                    Ok(core)
                }
            }
        }
    }

    /// Verify if the provided proxy address is indeed a user.
    pub fn assert_proxy(&self, maybe_proxy: &Addr) -> StdResult<Core> {
        let os_id = OS_ID
            .query(&self.deps.querier, maybe_proxy.clone())
            .map_err(|_| StdError::generic_err("Caller must be an OS proxy."))?;
        let maybe_os =
            OS_ADDRESSES.query(&self.deps.querier, self.base.registry(self.deps)?, os_id)?;
        match maybe_os {
            None => Err(StdError::generic_err(format!(
                "OS with id {} is not active.",
                os_id
            ))),
            Some(core) => {
                if &core.proxy != maybe_proxy {
                    Err(StdError::generic_err(
                        "Proposed proxy is not the proxy of this OS.",
                    ))
                } else {
                    Ok(core)
                }
            }
        }
    }
}
