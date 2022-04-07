use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;

use crate::native::memory::item::Memory;

use super::error::BaseDAppError;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
/// The BaseState contains the main addresses needed for sending and verifying messages
pub struct BaseState {
    pub proxy_address: Addr,
    pub traders: Vec<Addr>,
    pub memory: Memory,
}

impl BaseState {
    pub fn is_authorized_trader(&self, trader: &Addr) -> bool {
        self.traders.contains(trader)
    }

    /// Returns an Unauthorized Err if the provided trader is not authorized
    pub fn assert_authorized_trader(&self, trader: &Addr) -> Result<(), BaseDAppError> {
        if !self.is_authorized_trader(trader) {
            Err(BaseDAppError::Unauthorized {})
        } else {
            Ok(())
        }
    }
}

// Every DApp should use the provide memory contract for token/contract address resolution
pub const BASESTATE: Item<BaseState> = Item::new("\u{0}{10}base_state");
pub const ADMIN: Admin = Admin::new("admin");
