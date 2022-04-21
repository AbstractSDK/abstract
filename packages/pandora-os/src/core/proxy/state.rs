use cw_controllers::Admin;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Addr, CanonicalAddr, Decimal, Deps, Env, StdResult, Uint128};
use cw_storage_plus::{Item, Map};

use crate::core::proxy::proxy_assets::ProxyAsset;
use crate::queries::terraswap::query_pool;
use cw_asset::AssetInfo;
use terraswap::pair::PoolResponse;

use super::proxy_assets::{get_asset_identifier, get_tswap_asset_identifier};

pub static LUNA_DENOM: &str = "uluna";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct State {
    pub dapps: Vec<CanonicalAddr>,
}

pub const STATE: Item<State> = Item::new("\u{0}{5}state");
pub const ADMIN: Admin = Admin::new("admin");
pub const VAULT_ASSETS: Map<&str, ProxyAsset> = Map::new("proxy_assets");

