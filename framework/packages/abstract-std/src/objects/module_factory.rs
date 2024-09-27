use cosmwasm_std::{Addr, Api, CanonicalAddr, StdResult};

use crate::native_addrs;

/// Store the Module Factory contract.
#[cosmwasm_schema::cw_serde]
pub struct ModuleFactoryContract {
    /// Address of the module factory contract
    pub address: Addr,
}

impl ModuleFactoryContract {
    /// Retrieve address of the Version Control
    pub fn new(api: &dyn Api) -> StdResult<Self> {
        let address = api.addr_humanize(&CanonicalAddr::from(native_addrs::MODULE_FACTORY_ADDR))?;
        Ok(Self { address })
    }
}
