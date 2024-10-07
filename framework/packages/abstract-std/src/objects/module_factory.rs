use cosmwasm_std::{Addr, Api, Env};

use crate::{native_addrs, AbstractResult};

/// Store the Module Factory contract.
#[cosmwasm_schema::cw_serde]
pub struct ModuleFactoryContract {
    /// Address of the module factory contract
    pub address: Addr,
}

impl ModuleFactoryContract {
    /// Retrieve address of the Version Control
    pub fn new(api: &dyn Api, env: &Env) -> AbstractResult<Self> {
        let hrp = native_addrs::hrp_from_env(env);
        let address = api.addr_humanize(&native_addrs::module_factory_address(hrp, api)?)?;
        Ok(Self { address })
    }
}
