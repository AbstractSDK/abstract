use std::collections::HashSet;

use cosmwasm_std::{Addr, Coin, Timestamp};
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {}

pub type GradeName = String;
pub type GasPumpItem = (GradeName, Grade);

#[cosmwasm_schema::cw_serde]
pub struct Grade {
    /// The resolved mix of assets that make up this gas grade.
    pub fuel_mix: Vec<Coin>,
}

#[cosmwasm_schema::cw_serde]
pub struct GasPass {
    /// The grade of the gas.
    pub grade: GradeName,
    /// The expiration of the pass
    pub expiration: Option<Timestamp>
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const GRADES: Map<GradeName, Grade> = Map::new("grades");
pub const GAS_PASSES: Map<&Addr, GasPass> = Map::new("user_to_grade");
pub const GRADE_TO_USERS: Map<&GradeName, HashSet<Addr>> = Map::new("grade_to_users");
