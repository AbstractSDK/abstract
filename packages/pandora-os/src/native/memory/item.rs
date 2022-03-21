use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Deps, StdResult};

use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::queries::{
    query_asset_from_mem, query_assets_from_mem, query_contract_from_mem, query_contracts_from_mem,
};

// Struct that holds address
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Memory {
    pub address: Addr,
}

impl Memory {
    // Raw Query to Memory contract
    pub fn query_contracts(
        &self,
        deps: Deps,
        contract_names: &[String],
    ) -> StdResult<BTreeMap<String, Addr>> {
        query_contracts_from_mem(deps, &self.address, contract_names)
    }

    // Raw query of a single contract Addr
    pub fn query_contract(&self, deps: Deps, contract_name: &str) -> StdResult<Addr> {
        query_contract_from_mem(deps, &self.address, contract_name)
    }

    // Raw Query to Memory contract
    pub fn query_assets(
        &self,
        deps: Deps,
        asset_names: &[String],
    ) -> StdResult<BTreeMap<String, AssetInfo>> {
        query_assets_from_mem(deps, &self.address, asset_names)
    }

    // Raw query of a single AssetInfo
    pub fn query_asset(&self, deps: Deps, asset_name: &str) -> StdResult<AssetInfo> {
        query_asset_from_mem(deps, &self.address, asset_name)
    }
}
