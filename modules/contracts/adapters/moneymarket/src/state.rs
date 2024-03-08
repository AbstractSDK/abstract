use abstract_moneymarket_standard::msg::MoneymarketFees;
use cw_storage_plus::Item;

pub const MONEYMARKET_FEES: Item<MoneymarketFees> = Item::new("moneymarket_fees");
