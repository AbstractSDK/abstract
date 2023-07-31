use cw_storage_plus::Map;

pub const DOUBLE_DICE_OUTCOME: Map<&str, u8> = Map::new("double_dice_outcome");