use cw_asset::Asset;
use cw_storage_plus::{Item, Map};

#[cosmwasm_schema::cw_serde]
pub struct Config {}

pub type GasGrade = String;
#[cosmwasm_schema::cw_serde]
pub struct GasPump {
    pub denom: String,
    pub fuel_mix: Vec<Asset>,
}

pub const CONFIG: Item<Config> = Item::new("config");

pub const GAS_PUMPS: Map<GasGrade, GasPump> = Map::new("pumps");

/// Stores the temporary information on the pump before it is created
pub const PENDING_PUMP: Item<(GasGrade, GasPump)> = Item::new("pending_pump");
