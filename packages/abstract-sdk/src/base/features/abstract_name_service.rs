use abstract_os::objects::ans_host::AnsHost;
use cosmwasm_std::{Deps, StdResult};

/// Trait that enables APIs that depend on the Abstract Name System.
pub trait AbstractNameSystem: Sized {
    fn ans_host(&self, deps: Deps) -> StdResult<AnsHost>;
}
