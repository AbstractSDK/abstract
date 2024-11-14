use abstract_app::std::ibc::{Callback, IBCLifecycleComplete};
use cosmwasm_std::Binary;
use cw_storage_plus::{Item, Map};

pub const WINS: Item<u32> = Item::new("wins");
pub const LOSSES: Item<u32> = Item::new("losses");

pub const ICS20_CALLBACKS: Item<Vec<(Callback, IBCLifecycleComplete)>> =
    Item::new("ics20_callbacks");
