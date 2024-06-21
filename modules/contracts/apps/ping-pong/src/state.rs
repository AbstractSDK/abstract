use cw_storage_plus::Item;

pub const WINS: Item<u32> = Item::new("wins");
pub const LOSSES: Item<u32> = Item::new("losses");
