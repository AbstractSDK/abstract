use cw_storage_plus::Item;

pub const CURRENT_PONGS: Item<u32> = Item::new("pongs");
