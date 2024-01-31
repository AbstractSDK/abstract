use abstract_dex_standard::msg::DexFees;
use cw_storage_plus::Item;

pub const DEX_FEES: Item<DexFees> = Item::new("dex_fees");
