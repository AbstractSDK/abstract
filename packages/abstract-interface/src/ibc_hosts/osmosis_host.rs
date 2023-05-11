use abstract_core::ibc_host::*;
use cosmwasm_std::Empty;

use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, Empty, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl<Chain: CwEnv> OsmosisHost<Chain> {
    pub fn new(name: &str, chain: Chain) -> Self {
        Self(cw_orch::contract::Contract::new(name, chain))
    }
}
