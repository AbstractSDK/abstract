use abstract_core::ibc_host::*;
use cosmwasm_std::Empty;

use cw_orch::{interface, prelude::*};

#[interface(InstantiateMsg, Empty, QueryMsg, MigrateMsg)]
pub struct OsmosisHost<Chain>;

impl<Chain: CwEnv> OsmosisHost<Chain> {}
