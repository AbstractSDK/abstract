use abstract_app::objects::chain_name::ChainName;
use cw_storage_plus::Item;

pub const CURRENT_PONGS: Item<u32> = Item::new("pongs");
pub const PREVIOUS_PING_PONG: Item<(u32, ChainName)> = Item::new("ppp");
