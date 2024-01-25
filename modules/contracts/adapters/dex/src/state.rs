use abstract_core::objects::fee::UsageFee;
use cw_storage_plus::Item;

pub const SWAP_FEE: Item<UsageFee> = Item::new("swap_fee");
