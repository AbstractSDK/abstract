use abstract_money_market_standard::msg::MoneymarketFees;
use cw_storage_plus::Item;

pub const MONEYMARKET_FEES: Item<MoneymarketFees> = Item::new("money_market_fees");
