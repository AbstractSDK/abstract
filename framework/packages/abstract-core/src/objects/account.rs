mod account_id;
mod account_trace;

use cw_storage_plus::Item;

pub use self::{account_id::AccountId, account_trace::AccountTrace};

pub const ABSTRACT_ACCOUNT_ID: AccountId = AccountId::const_new(0, AccountTrace::Local);

/// Identifier for a chain
/// Example: "juno", "terra", "osmosis", ...
pub type ChainId = String;
pub type AccountSequence = u32;

/// Account Id storage key
pub const ACCOUNT_ID: Item<AccountId> = Item::new("acc_id");
