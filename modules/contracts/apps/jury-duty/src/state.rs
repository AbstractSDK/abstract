use cosmwasm_std::Addr;
use cw_storage_plus::Map;

pub const EXEMPTIONS: Map<&u64, Vec<Addr>> = Map::new("exempt");
pub const JURIES: Map<&u64, Vec<String>> = Map::new("jury");
