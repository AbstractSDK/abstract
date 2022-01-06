use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use crate::memory::item::Memory;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// The BaseState contains the main addresses needed for sending and verifying messages
pub struct BaseState {
    pub treasury_address: Addr,
    pub trader: Addr,
    pub memory: Memory,
}

// Every DApp should use the provide memory contract for token/contract address resolution
pub const BASESTATE: Item<BaseState> = Item::new("\u{0}{10}base_state");
pub const ADMIN: Admin = Admin::new("admin");
