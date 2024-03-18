use abstract_core::objects::fee::UsageFee;
use cw_storage_plus::Item;

pub const MONEYMARKET_FEES: Item<UsageFee> = Item::new("money_market_fees");
