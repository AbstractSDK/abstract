use abstract_money_market_standard::msg::MoneyMarketFees;
use cw_storage_plus::Item;

pub const MONEYMARKET_FEES: Item<MoneyMarketFees> = Item::new("money_market_fees");
