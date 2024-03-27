use abstract_core::objects::fee::UsageFee;
use cw_storage_plus::Item;

pub const MONEY_MARKET_FEES: Item<UsageFee> = Item::new("money_market_fees");
