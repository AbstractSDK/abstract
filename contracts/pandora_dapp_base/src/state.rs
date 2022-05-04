use std::marker::PhantomData;

use cosmwasm_std::Addr;
use cw_controllers::Admin;
use cw_storage_plus::Item;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

use pandora_os::native::memory::item::Memory;
use pandora_os::pandora_dapp::constants::{ADMIN_KEY, BASE_STATE_KEY};
use pandora_os::pandora_dapp::traits::{CustomMsg, Dapp};

use crate::DappError;

/// The state variables for our DappContract.
pub struct DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    // Every DApp should use the provided memory contract for token/contract address resolution
    pub base_state: Item<'a, DappState>,
    pub admin: Admin<'a>,
    pub extension: Item<'a, T>,

    pub(crate) _custom_response: PhantomData<C>,
}

/// This is simply an "interface", the implementations are in other files
impl<'a, T, C> Dapp<T, C> for DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
    C: CustomMsg,
{
}

impl<T, C> Default for DappContract<'static, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn default() -> Self {
        Self::new(BASE_STATE_KEY, ADMIN_KEY)
    }
}

/// Constructor
impl<'a, T, C> DappContract<'a, T, C>
where
    T: Serialize + DeserializeOwned + Clone,
{
    fn new(base_state_key: &'a str, admin_key: &'a str) -> Self {
        Self {
            // contract_info: Item::new(contract_key),
            base_state: Item::new(base_state_key),
            admin: Admin::new(admin_key),
            extension: Item::new("TODO_SOMETHING_HERE"),
            _custom_response: PhantomData,
        }
    }
}

/// The BaseState contains the main addresses needed for sending and verifying messages
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct DappState {
    /// Proxy contract address for relaying transactions
    pub proxy_address: Addr,
    /// Memory contract struct (address)
    pub memory: Memory,
    /// Authorized users for the dapp. TODO: should we support expiring perms?
    pub traders: Vec<Addr>,
}

impl DappState {
    pub fn is_authorized_trader(&self, trader: &Addr) -> bool {
        self.traders.contains(trader)
    }

    /// Returns an Unauthorized Err if the provided trader is not authorized
    pub fn assert_authorized_trader(&self, trader: &Addr) -> Result<(), DappError> {
        if !self.is_authorized_trader(trader) {
            Err(DappError::Unauthorized {})
        } else {
            Ok(())
        }
    }
}
