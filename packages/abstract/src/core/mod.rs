pub mod manager;
pub mod modules;
pub mod proxy;

pub mod common {
    use cosmwasm_std::{Addr, QuerierWrapper, StdResult};
    use cw_storage_plus::Item;

    pub const OS_ID: Item<u32> = Item::new("\u{0}{5}os_id");

    /// Query the OS id
    pub fn query_os_id(querier: &QuerierWrapper, core_contract_addr: &Addr) -> StdResult<u32> {
        OS_ID.query(querier, core_contract_addr.clone())
    }
}
