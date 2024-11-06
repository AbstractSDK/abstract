use cosmwasm_std::{Addr, Api, Deps, Env};

use crate::{native_addrs, AbstractResult};

/// Store the Module Factory contract.
#[cosmwasm_schema::cw_serde]
pub struct ModuleFactoryContract {
    /// Address of the module factory contract
    pub address: Addr,
}

impl ModuleFactoryContract {
    /// Retrieve address of the Registry
    pub fn new(deps: Deps, account_code_id: u64) -> AbstractResult<Self> {
        let address = deps
            .api
            .addr_humanize(&native_addrs::module_factory_address(
                deps,
                account_code_id,
            )?)?;
        Ok(Self { address })
    }
}
