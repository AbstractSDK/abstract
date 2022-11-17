use abstract_os::{
    objects::common_namespace::ADMIN_NAMESPACE, proxy::state::OS_ID, version_control::Core,
};
use cosmwasm_std::{Addr, Deps, StdError, StdResult};
use cw_storage_plus::Item;

const MANAGER: Item<'_, Option<Addr>> = Item::new(ADMIN_NAMESPACE);

pub trait Identification: Sized {
    fn proxy_address(&self, deps: Deps) -> StdResult<Addr>;
    fn manager_address(&self, deps: Deps) -> StdResult<Addr> {
        let maybe_proxy_manager = MANAGER.query(&deps.querier, self.proxy_address(deps)?)?;
        maybe_proxy_manager.ok_or_else(|| StdError::generic_err("proxy admin must be manager."))
    }
    fn os_core(&self, deps: Deps) -> StdResult<Core> {
        Ok(Core {
            manager: self.manager_address(deps)?,
            proxy: self.proxy_address(deps)?,
        })
    }
    fn os_id(&self, deps: Deps) -> StdResult<u32> {
        OS_ID.query(&deps.querier, self.proxy_address(deps)?)
    }
}
