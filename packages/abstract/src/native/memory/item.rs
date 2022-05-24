use std::collections::BTreeMap;

use cosmwasm_std::{Addr, Deps, StdResult};

use cw_asset::AssetInfo;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use super::{
    queries::{
        query_asset_from_mem, query_assets_from_mem, query_contract_from_mem,
        query_contracts_from_mem,
    },
    state::PAIR_POSTFIX,
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

    /// Query single pair address from mem
    pub fn query_pair_address(
        &self,
        deps: Deps,
        asset_names: [String; 2],
        dex: &str,
    ) -> StdResult<Addr> {
        let mut lowercase = asset_names.map(|s| s.to_ascii_lowercase());
        lowercase.sort();
        let key = format!("{}_{}_{}_{}", dex, lowercase[0], lowercase[1], PAIR_POSTFIX);
        query_contract_from_mem(deps, &self.address, &key)
    }
}
