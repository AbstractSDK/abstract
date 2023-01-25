//! # Feature Objects
//! Feature objects are objects that store sufficient data to unlock a set of APIs.
//! These objects are mostly used internally to easy re-use application code without
//! requiring the usage of a base contract.  

use abstract_os::version_control::Core;
use cosmwasm_std::{Addr, Deps};

use crate::apis::ModuleIdentification;
use crate::base::features::{Identification, RegisterAccess};
pub use abstract_os::objects::ans_host::AnsHost;
use os::PROXY;

#[derive(Clone)]
/// Store the Version Control contract.
/// Implements [`RegisterAccess`]
pub struct VersionControlContract {
    pub address: Addr,
}

impl RegisterAccess for VersionControlContract {
    fn registry(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(self.address.clone())
    }
}

#[derive(Clone)]
/// Store a proxy contract address.
/// Implements [`Identification`].
pub struct ProxyContract {
    pub contract_address: Addr,
}

impl Identification for ProxyContract {
    fn proxy_address(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(self.contract_address.clone())
    }
}

impl ModuleIdentification for ProxyContract {
    fn module_id(&self) -> &'static str {
        PROXY
    }
}

impl Identification for Core {
    fn proxy_address(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(self.proxy.clone())
    }

    fn manager_address(&self, _deps: Deps) -> cosmwasm_std::StdResult<Addr> {
        Ok(self.manager.clone())
    }

    fn os_core(&self, _deps: Deps) -> cosmwasm_std::StdResult<Core> {
        Ok(self.clone())
    }
}

impl ModuleIdentification for Core {
    // Any actions executed by the core will be by the proxy address
    fn module_id(&self) -> &'static str {
        PROXY
    }
}

impl crate::base::features::AbstractNameService for AnsHost {
    fn ans_host(
        &self,
        _deps: Deps,
    ) -> cosmwasm_std::StdResult<abstract_os::objects::ans_host::AnsHost> {
        Ok(self.clone())
    }
}
